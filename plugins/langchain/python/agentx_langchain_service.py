#!/usr/bin/env python3
"""
AgentX LangChain服务
提供LangChain框架的HTTP API接口，供Rust插件调用
"""

import asyncio
import json
import logging
import os
import sys
from typing import Dict, List, Any, Optional
from datetime import datetime

import uvicorn
from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field

# LangChain imports
try:
    from langchain.llms import OpenAI
    from langchain.chat_models import ChatOpenAI
    from langchain.agents import initialize_agent, AgentType
    from langchain.tools import Tool
    from langchain.memory import ConversationBufferMemory
    from langchain.chains import LLMChain, ConversationChain
    from langchain.prompts import PromptTemplate
    from langchain.schema import HumanMessage, AIMessage, SystemMessage
    from langchain.callbacks.manager import CallbackManager
    from langchain.callbacks.streaming_stdout import StreamingStdOutCallbackHandler
except ImportError as e:
    print(f"错误: 无法导入LangChain模块: {e}")
    print("请安装LangChain: pip install langchain langchain-community langchain-core")
    sys.exit(1)

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# FastAPI应用
app = FastAPI(
    title="AgentX LangChain Service",
    description="LangChain框架HTTP API服务",
    version="1.0.0"
)

# CORS中间件
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# 全局状态
class ServiceState:
    def __init__(self):
        self.agents: Dict[str, Any] = {}
        self.models: Dict[str, Any] = {}
        self.tools: Dict[str, Tool] = {}
        self.sessions: Dict[str, Any] = {}
        self.initialized = False

state = ServiceState()

# 请求/响应模型
class ChatRequest(BaseModel):
    agent_type: str = "conversational"
    model: str = "gpt-3.5-turbo"
    messages: List[Dict[str, str]]
    tools: Optional[List[str]] = None
    memory_type: Optional[str] = None
    config: Optional[Dict[str, Any]] = None

class ChatResponse(BaseModel):
    content: str
    usage: Optional[Dict[str, Any]] = None
    model: str
    timestamp: str

class ToolRequest(BaseModel):
    tool_name: str
    arguments: Dict[str, Any]
    agent_config: Optional[Dict[str, Any]] = None

class ToolResponse(BaseModel):
    result: Any
    success: bool
    error: Optional[str] = None

class ChainRequest(BaseModel):
    chain_config: Dict[str, Any]
    agent_config: Optional[Dict[str, Any]] = None
    model: str = "gpt-3.5-turbo"

class ChainResponse(BaseModel):
    result: Any
    success: bool
    error: Optional[str] = None

class AgentCreateRequest(BaseModel):
    agent_id: str
    agent_type: str = "conversational"
    model: str = "gpt-3.5-turbo"
    tools: Optional[List[str]] = None
    memory_type: Optional[str] = None
    config: Optional[Dict[str, Any]] = None

class VersionResponse(BaseModel):
    python_version: str
    langchain_version: str
    service_version: str

class PackageCheckRequest(BaseModel):
    package: str

class PackageCheckResponse(BaseModel):
    installed: bool
    version: Optional[str] = None

# 工具函数
def get_openai_api_key() -> str:
    """获取OpenAI API密钥"""
    api_key = os.getenv("OPENAI_API_KEY")
    if not api_key:
        raise HTTPException(status_code=500, detail="OPENAI_API_KEY环境变量未设置")
    return api_key

def create_chat_model(model_name: str, **kwargs) -> Any:
    """创建聊天模型"""
    try:
        if model_name.startswith("gpt-"):
            return ChatOpenAI(
                model_name=model_name,
                openai_api_key=get_openai_api_key(),
                **kwargs
            )
        else:
            raise ValueError(f"不支持的模型: {model_name}")
    except Exception as e:
        logger.error(f"创建模型失败: {e}")
        raise HTTPException(status_code=500, detail=f"创建模型失败: {e}")

def create_basic_tools() -> List[Tool]:
    """创建基础工具"""
    def calculator(expression: str) -> str:
        """简单计算器工具"""
        try:
            # 安全的数学表达式计算
            allowed_chars = set("0123456789+-*/.() ")
            if not all(c in allowed_chars for c in expression):
                return "错误: 包含不允许的字符"
            result = eval(expression)
            return str(result)
        except Exception as e:
            return f"计算错误: {e}"
    
    def search_tool(query: str) -> str:
        """模拟搜索工具"""
        return f"搜索结果: 关于'{query}'的信息（这是一个模拟结果）"
    
    def weather_tool(location: str) -> str:
        """模拟天气工具"""
        return f"{location}的天气: 晴天，温度25°C（这是一个模拟结果）"
    
    return [
        Tool(
            name="calculator",
            description="用于数学计算的工具",
            func=calculator
        ),
        Tool(
            name="search",
            description="用于搜索信息的工具",
            func=search_tool
        ),
        Tool(
            name="weather",
            description="用于获取天气信息的工具",
            func=weather_tool
        )
    ]

# API端点
@app.get("/health")
async def health_check():
    """健康检查"""
    return {
        "status": "healthy",
        "timestamp": datetime.now().isoformat(),
        "initialized": state.initialized
    }

@app.get("/version", response_model=VersionResponse)
async def get_version():
    """获取版本信息"""
    import langchain
    return VersionResponse(
        python_version=sys.version,
        langchain_version=langchain.__version__,
        service_version="1.0.0"
    )

@app.post("/check_package", response_model=PackageCheckResponse)
async def check_package(request: PackageCheckRequest):
    """检查Python包是否已安装"""
    try:
        import importlib
        module = importlib.import_module(request.package)
        version = getattr(module, '__version__', None)
        return PackageCheckResponse(installed=True, version=version)
    except ImportError:
        return PackageCheckResponse(installed=False)

@app.post("/initialize")
async def initialize_service():
    """初始化LangChain环境"""
    try:
        logger.info("初始化LangChain服务...")
        
        # 创建基础工具
        state.tools.update({tool.name: tool for tool in create_basic_tools()})
        
        # 预加载常用模型
        common_models = ["gpt-3.5-turbo"]
        for model_name in common_models:
            try:
                model = create_chat_model(model_name)
                state.models[model_name] = model
                logger.info(f"预加载模型: {model_name}")
            except Exception as e:
                logger.warning(f"预加载模型 {model_name} 失败: {e}")
        
        state.initialized = True
        logger.info("LangChain服务初始化完成")
        
        return {"success": True, "message": "初始化成功"}
    except Exception as e:
        logger.error(f"初始化失败: {e}")
        raise HTTPException(status_code=500, detail=f"初始化失败: {e}")

@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    """处理聊天请求"""
    try:
        logger.info(f"处理聊天请求: {request.model}")
        
        # 获取或创建模型
        if request.model not in state.models:
            state.models[request.model] = create_chat_model(request.model)
        
        model = state.models[request.model]
        
        # 构建消息
        messages = []
        for msg in request.messages:
            if msg["role"] == "user":
                messages.append(HumanMessage(content=msg["content"]))
            elif msg["role"] == "assistant":
                messages.append(AIMessage(content=msg["content"]))
            elif msg["role"] == "system":
                messages.append(SystemMessage(content=msg["content"]))
        
        # 生成响应
        response = model(messages)
        
        return ChatResponse(
            content=response.content,
            model=request.model,
            timestamp=datetime.now().isoformat()
        )
    except Exception as e:
        logger.error(f"聊天处理失败: {e}")
        raise HTTPException(status_code=500, detail=f"聊天处理失败: {e}")

@app.post("/tool", response_model=ToolResponse)
async def call_tool(request: ToolRequest):
    """调用工具"""
    try:
        logger.info(f"调用工具: {request.tool_name}")
        
        if request.tool_name not in state.tools:
            return ToolResponse(
                result=None,
                success=False,
                error=f"工具 '{request.tool_name}' 不存在"
            )
        
        tool = state.tools[request.tool_name]
        
        # 构建工具输入
        if len(request.arguments) == 1:
            # 单参数工具
            input_value = list(request.arguments.values())[0]
        else:
            # 多参数工具，转换为字符串
            input_value = json.dumps(request.arguments)
        
        # 调用工具
        result = tool.func(input_value)
        
        return ToolResponse(
            result=result,
            success=True
        )
    except Exception as e:
        logger.error(f"工具调用失败: {e}")
        return ToolResponse(
            result=None,
            success=False,
            error=str(e)
        )

@app.post("/chain", response_model=ChainResponse)
async def execute_chain(request: ChainRequest):
    """执行LangChain链"""
    try:
        logger.info("执行LangChain链")
        
        # 获取模型
        if request.model not in state.models:
            state.models[request.model] = create_chat_model(request.model)
        
        model = state.models[request.model]
        
        # 根据链配置创建链
        chain_type = request.chain_config.get("type", "llm")
        
        if chain_type == "llm":
            # 简单LLM链
            prompt_template = request.chain_config.get("prompt", "{input}")
            prompt = PromptTemplate(
                input_variables=["input"],
                template=prompt_template
            )
            chain = LLMChain(llm=model, prompt=prompt)
            
            input_data = request.chain_config.get("input", "")
            result = chain.run(input=input_data)
        
        elif chain_type == "conversation":
            # 对话链
            memory = ConversationBufferMemory()
            chain = ConversationChain(llm=model, memory=memory)
            
            input_data = request.chain_config.get("input", "")
            result = chain.run(input=input_data)
        
        else:
            return ChainResponse(
                result=None,
                success=False,
                error=f"不支持的链类型: {chain_type}"
            )
        
        return ChainResponse(
            result=result,
            success=True
        )
    except Exception as e:
        logger.error(f"链执行失败: {e}")
        return ChainResponse(
            result=None,
            success=False,
            error=str(e)
        )

@app.post("/agent/create")
async def create_agent(request: AgentCreateRequest):
    """创建Agent"""
    try:
        logger.info(f"创建Agent: {request.agent_id}")
        
        # 获取模型
        if request.model not in state.models:
            state.models[request.model] = create_chat_model(request.model)
        
        model = state.models[request.model]
        
        # 创建Agent
        if request.agent_type == "conversational":
            # 对话型Agent
            memory = ConversationBufferMemory()
            agent_data = {
                "type": "conversational",
                "model": model,
                "memory": memory,
                "tools": request.tools or [],
                "config": request.config or {}
            }
        
        elif request.agent_type == "tool_using":
            # 工具使用Agent
            tools = [state.tools[tool_name] for tool_name in (request.tools or []) if tool_name in state.tools]
            agent = initialize_agent(
                tools=tools,
                llm=model,
                agent=AgentType.ZERO_SHOT_REACT_DESCRIPTION,
                verbose=True
            )
            agent_data = {
                "type": "tool_using",
                "agent": agent,
                "tools": request.tools or [],
                "config": request.config or {}
            }
        
        else:
            raise ValueError(f"不支持的Agent类型: {request.agent_type}")
        
        # 保存Agent
        state.agents[request.agent_id] = agent_data
        
        return {"success": True, "agent_id": request.agent_id}
    except Exception as e:
        logger.error(f"创建Agent失败: {e}")
        raise HTTPException(status_code=500, detail=f"创建Agent失败: {e}")

@app.post("/agent/delete")
async def delete_agent(request: dict):
    """删除Agent"""
    try:
        agent_id = request.get("agent_id")
        if not agent_id:
            raise HTTPException(status_code=400, detail="缺少agent_id")
        
        if agent_id in state.agents:
            del state.agents[agent_id]
            logger.info(f"删除Agent: {agent_id}")
        
        return {"success": True}
    except Exception as e:
        logger.error(f"删除Agent失败: {e}")
        raise HTTPException(status_code=500, detail=f"删除Agent失败: {e}")

@app.post("/model/preload")
async def preload_model(request: dict):
    """预加载模型"""
    try:
        model_name = request.get("model")
        if not model_name:
            raise HTTPException(status_code=400, detail="缺少model参数")
        
        if model_name not in state.models:
            state.models[model_name] = create_chat_model(model_name)
            logger.info(f"预加载模型: {model_name}")
        
        return {"success": True, "model": model_name}
    except Exception as e:
        logger.error(f"预加载模型失败: {e}")
        raise HTTPException(status_code=500, detail=f"预加载模型失败: {e}")

@app.post("/tool/preload")
async def preload_tool(request: dict):
    """预加载工具"""
    try:
        tool_name = request.get("tool")
        if not tool_name:
            raise HTTPException(status_code=400, detail="缺少tool参数")
        
        # 工具已在初始化时加载
        if tool_name in state.tools:
            logger.info(f"工具已存在: {tool_name}")
        else:
            logger.warning(f"工具不存在: {tool_name}")
        
        return {"success": True, "tool": tool_name}
    except Exception as e:
        logger.error(f"预加载工具失败: {e}")
        raise HTTPException(status_code=500, detail=f"预加载工具失败: {e}")

def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description="AgentX LangChain Service")
    parser.add_argument("--host", default="127.0.0.1", help="服务器主机")
    parser.add_argument("--port", type=int, default=8000, help="服务器端口")
    parser.add_argument("--log-level", default="info", help="日志级别")
    
    args = parser.parse_args()
    
    # 设置日志级别
    logging.getLogger().setLevel(getattr(logging, args.log_level.upper()))
    
    logger.info(f"启动AgentX LangChain服务在 {args.host}:{args.port}")
    
    # 启动服务器
    uvicorn.run(
        app,
        host=args.host,
        port=args.port,
        log_level=args.log_level
    )

if __name__ == "__main__":
    main()
