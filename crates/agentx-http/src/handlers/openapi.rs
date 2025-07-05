//! OpenAPI文档处理器
//! 
//! 提供OpenAPI 3.0规范和Swagger UI文档

use axum::{
    response::{Html, Json, Response},
    http::{header, StatusCode},
    body::Body,
};
use serde_json::{json, Value};

use crate::error::HttpApiResult;

/// 获取OpenAPI 3.0规范
pub async fn get_openapi_spec() -> HttpApiResult<Json<Value>> {
    let spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "AgentX HTTP API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "AgentX AI Agent互操作平台HTTP API\n\n这是一个基于A2A协议的AI Agent通信平台，提供Agent注册、消息路由、任务管理等功能。",
            "contact": {
                "name": "AgentX Team",
                "url": "https://github.com/agentx/agentx",
                "email": "team@agentx.dev"
            },
            "license": {
                "name": "Apache 2.0",
                "url": "https://www.apache.org/licenses/LICENSE-2.0.html"
            }
        },
        "servers": [
            {
                "url": "/api/v1",
                "description": "API v1"
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "健康检查",
                    "description": "检查服务器和A2A引擎的健康状态",
                    "operationId": "healthCheck",
                    "tags": ["System"],
                    "responses": {
                        "200": {
                            "description": "健康检查结果",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/HealthResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/metrics": {
                "get": {
                    "summary": "获取系统指标",
                    "description": "获取详细的系统性能和使用指标",
                    "operationId": "getMetrics",
                    "tags": ["Monitoring"],
                    "responses": {
                        "200": {
                            "description": "系统指标数据",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/MetricsResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/metrics/prometheus": {
                "get": {
                    "summary": "获取Prometheus格式指标",
                    "description": "获取Prometheus监控系统兼容的指标数据",
                    "operationId": "getPrometheusMetrics",
                    "tags": ["Monitoring"],
                    "responses": {
                        "200": {
                            "description": "Prometheus格式指标",
                            "content": {
                                "text/plain": {
                                    "schema": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/agents": {
                "get": {
                    "summary": "列出所有Agent",
                    "description": "获取已注册的所有Agent列表",
                    "operationId": "listAgents",
                    "tags": ["Agents"],
                    "parameters": [
                        {
                            "name": "limit",
                            "in": "query",
                            "description": "返回结果的最大数量",
                            "schema": {
                                "type": "integer",
                                "minimum": 1,
                                "maximum": 1000,
                                "default": 100
                            }
                        },
                        {
                            "name": "offset",
                            "in": "query",
                            "description": "跳过的结果数量",
                            "schema": {
                                "type": "integer",
                                "minimum": 0,
                                "default": 0
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Agent列表",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/AgentListResponse"
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "注册新Agent",
                    "description": "向系统注册一个新的Agent",
                    "operationId": "registerAgent",
                    "tags": ["Agents"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/RegisterAgentRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Agent注册成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/AgentResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "请求参数错误"
                        },
                        "409": {
                            "description": "Agent已存在"
                        }
                    }
                }
            },
            "/agents/{agentId}": {
                "get": {
                    "summary": "获取特定Agent",
                    "description": "根据ID获取特定Agent的详细信息",
                    "operationId": "getAgent",
                    "tags": ["Agents"],
                    "parameters": [
                        {
                            "name": "agentId",
                            "in": "path",
                            "required": true,
                            "description": "Agent的唯一标识符",
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Agent详细信息",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/AgentResponse"
                                    }
                                }
                            }
                        },
                        "404": {
                            "description": "Agent不存在"
                        }
                    }
                },
                "delete": {
                    "summary": "注销Agent",
                    "description": "从系统中注销指定的Agent",
                    "operationId": "unregisterAgent",
                    "tags": ["Agents"],
                    "parameters": [
                        {
                            "name": "agentId",
                            "in": "path",
                            "required": true,
                            "description": "Agent的唯一标识符",
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Agent注销成功"
                        },
                        "404": {
                            "description": "Agent不存在"
                        }
                    }
                }
            },
            "/messages": {
                "post": {
                    "summary": "发送消息",
                    "description": "向系统发送A2A消息",
                    "operationId": "sendMessage",
                    "tags": ["Messages"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/SendMessageRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "消息发送成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/MessageResponse"
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "消息格式错误"
                        },
                        "500": {
                            "description": "消息路由失败"
                        }
                    }
                }
            },
            "/tasks": {
                "get": {
                    "summary": "列出所有任务",
                    "description": "获取系统中的所有任务",
                    "operationId": "listTasks",
                    "tags": ["Tasks"],
                    "responses": {
                        "200": {
                            "description": "任务列表",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/TaskListResponse"
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "创建新任务",
                    "description": "创建一个新的A2A任务",
                    "operationId": "createTask",
                    "tags": ["Tasks"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/CreateTaskRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "任务创建成功",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/TaskResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "ApiResponse": {
                    "type": "object",
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "description": "请求是否成功"
                        },
                        "data": {
                            "description": "响应数据"
                        },
                        "error": {
                            "type": "string",
                            "description": "错误信息（仅在失败时存在）"
                        },
                        "timestamp": {
                            "type": "string",
                            "format": "date-time",
                            "description": "响应时间戳"
                        },
                        "request_id": {
                            "type": "string",
                            "description": "请求唯一标识符"
                        }
                    },
                    "required": ["success", "timestamp", "request_id"]
                },
                "HealthResponse": {
                    "allOf": [
                        {
                            "$ref": "#/components/schemas/ApiResponse"
                        },
                        {
                            "type": "object",
                            "properties": {
                                "data": {
                                    "type": "object",
                                    "properties": {
                                        "status": {
                                            "type": "string",
                                            "enum": ["healthy", "unhealthy"]
                                        },
                                        "checks": {
                                            "type": "object"
                                        },
                                        "uptime": {
                                            "type": "string"
                                        },
                                        "version": {
                                            "type": "string"
                                        }
                                    }
                                }
                            }
                        }
                    ]
                },
                "Agent": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Agent唯一标识符"
                        },
                        "name": {
                            "type": "string",
                            "description": "Agent名称"
                        },
                        "description": {
                            "type": "string",
                            "description": "Agent描述"
                        },
                        "version": {
                            "type": "string",
                            "description": "Agent版本"
                        },
                        "capabilities": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Agent能力列表"
                        },
                        "status": {
                            "type": "string",
                            "enum": ["active", "inactive", "error"]
                        },
                        "created_at": {
                            "type": "string",
                            "format": "date-time"
                        },
                        "updated_at": {
                            "type": "string",
                            "format": "date-time"
                        }
                    },
                    "required": ["id", "name", "version"]
                },
                "RegisterAgentRequest": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Agent唯一标识符"
                        },
                        "name": {
                            "type": "string",
                            "description": "Agent名称"
                        },
                        "description": {
                            "type": "string",
                            "description": "Agent描述"
                        },
                        "version": {
                            "type": "string",
                            "description": "Agent版本"
                        },
                        "capabilities": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Agent能力列表"
                        },
                        "endpoint": {
                            "type": "string",
                            "format": "uri",
                            "description": "Agent服务端点"
                        }
                    },
                    "required": ["id", "name", "version", "endpoint"]
                },
                "SendMessageRequest": {
                    "type": "object",
                    "properties": {
                        "role": {
                            "type": "string",
                            "enum": ["User", "Agent"],
                            "description": "消息角色"
                        },
                        "content": {
                            "type": "string",
                            "description": "消息内容"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "消息元数据"
                        },
                        "target_agent": {
                            "type": "string",
                            "description": "目标Agent ID（可选）"
                        }
                    },
                    "required": ["role", "content"]
                }
            },
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key"
                },
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer"
                }
            }
        },
        "tags": [
            {
                "name": "System",
                "description": "系统管理相关接口"
            },
            {
                "name": "Monitoring",
                "description": "监控和指标相关接口"
            },
            {
                "name": "Agents",
                "description": "Agent管理相关接口"
            },
            {
                "name": "Messages",
                "description": "消息处理相关接口"
            },
            {
                "name": "Tasks",
                "description": "任务管理相关接口"
            }
        ]
    });
    
    Ok(Json(spec))
}

/// 获取Swagger UI文档页面
pub async fn get_swagger_ui() -> HttpApiResult<Html<String>> {
    let html = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AgentX API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
    <style>
        html {
            box-sizing: border-box;
            overflow: -moz-scrollbars-vertical;
            overflow-y: scroll;
        }
        *, *:before, *:after {
            box-sizing: inherit;
        }
        body {
            margin:0;
            background: #fafafa;
        }
        .swagger-ui .topbar {
            background-color: #1976d2;
        }
        .swagger-ui .topbar .download-url-wrapper .select-label {
            color: #fff;
        }
        .swagger-ui .topbar .download-url-wrapper input[type=text] {
            border: 2px solid #1976d2;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js"></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-standalone-preset.js"></script>
    <script>
        window.onload = function() {
            const ui = SwaggerUIBundle({
                url: '/api/v1/openapi.json',
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                plugins: [
                    SwaggerUIBundle.plugins.DownloadUrl
                ],
                layout: "StandaloneLayout",
                validatorUrl: null,
                tryItOutEnabled: true,
                supportedSubmitMethods: ['get', 'post', 'put', 'delete', 'patch'],
                onComplete: function() {
                    console.log('AgentX API Documentation loaded');
                },
                requestInterceptor: function(request) {
                    // 可以在这里添加认证头
                    return request;
                },
                responseInterceptor: function(response) {
                    return response;
                }
            });
            
            window.ui = ui;
        };
    </script>
</body>
</html>
    "#;
    
    Ok(Html(html.to_string()))
}

/// 获取ReDoc文档页面
pub async fn get_redoc() -> HttpApiResult<Html<String>> {
    let html = r#"
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AgentX API Documentation - ReDoc</title>
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
    <style>
        body {
            margin: 0;
            padding: 0;
        }
    </style>
</head>
<body>
    <redoc spec-url='/api/v1/openapi.json'></redoc>
    <script src="https://cdn.jsdelivr.net/npm/redoc@2.1.3/bundles/redoc.standalone.js"></script>
</body>
</html>
    "#;
    
    Ok(Html(html.to_string()))
}

/// 下载OpenAPI规范文件
pub async fn download_openapi_spec() -> HttpApiResult<Response<Body>> {
    let spec = get_openapi_spec().await?;
    let json_content = serde_json::to_string_pretty(&spec.0)?;
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"agentx-openapi.json\"")
        .body(Body::from(json_content))
        .map_err(|e| crate::error::HttpApiError::InternalError(format!("构建响应失败: {}", e)))?;
    
    Ok(response)
}
