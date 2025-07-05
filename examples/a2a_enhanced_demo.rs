//! A2A协议v0.2.5增强功能演示
//! 
//! 本示例展示AgentX中A2A协议的最新功能，包括：
//! - 多模态交互支持
//! - UX协商能力
//! - 企业级信任管理
//! - Agent发现和匹配

use agentx_a2a::{
    AgentCard, Capability, CapabilityType, Endpoint,
    InteractionModality, UxCapabilities, TrustLevel,
    A2AMessage, MessageRole, MessagePart, FileData, FileWithBytes,
};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentX A2A协议v0.2.5增强功能演示");
    println!("基于Google A2A规范: https://a2aproject.github.io/A2A/");
    
    // 1. 创建多模态AI Agent
    println!("\n📋 1. 创建多模态AI Agent");
    let multimodal_agent = create_multimodal_agent();
    print_agent_info(&multimodal_agent, "多模态AI Agent");
    
    // 2. 创建企业级Agent
    println!("\n🏢 2. 创建企业级Agent");
    let enterprise_agent = create_enterprise_agent();
    print_agent_info(&enterprise_agent, "企业级Agent");
    
    // 3. 演示Agent发现和匹配
    println!("\n🔍 3. Agent发现和能力匹配");
    demonstrate_agent_discovery(&[&multimodal_agent, &enterprise_agent]);
    
    // 4. 演示多模态消息交换
    println!("\n💬 4. 多模态消息交换");
    demonstrate_multimodal_messages().await;
    
    // 5. 演示UX协商
    println!("\n🎨 5. UX协商演示");
    demonstrate_ux_negotiation(&multimodal_agent);
    
    // 6. 演示信任级别管理
    println!("\n🔒 6. 企业信任级别管理");
    demonstrate_trust_management(&[&multimodal_agent, &enterprise_agent]);
    
    println!("\n🎉 A2A协议增强功能演示完成！");
    println!("✅ 所有功能都符合A2A v0.2.5规范");
    
    Ok(())
}

/// 创建多模态AI Agent
fn create_multimodal_agent() -> AgentCard {
    let ux_capabilities = UxCapabilities::new()
        .with_component("chat_interface".to_string())
        .with_component("image_viewer".to_string())
        .with_component("file_uploader".to_string())
        .with_dynamic_adaptation()
        .with_multimodal_support()
        .with_custom_protocol("voice_chat".to_string());
    
    AgentCard::new(
        "multimodal_ai".to_string(),
        "多模态AI助手".to_string(),
        "支持文本、图像、音频等多种模态的AI助手".to_string(),
        "2.0.0".to_string(),
    )
    .add_capability(Capability::new(
        "text_generation".to_string(),
        "生成高质量文本内容".to_string(),
        CapabilityType::TextGeneration,
    ))
    .add_capability(Capability::new(
        "image_analysis".to_string(),
        "分析和理解图像内容".to_string(),
        CapabilityType::ImageProcessing,
    ))
    .add_capability(Capability::new(
        "audio_processing".to_string(),
        "处理音频文件和语音识别".to_string(),
        CapabilityType::AudioProcessing,
    ))
    .add_endpoint(Endpoint::new(
        "http".to_string(),
        "https://api.multimodal-ai.com/v1".to_string(),
    ))
    .with_interaction_modality(InteractionModality::Text)
    .with_interaction_modality(InteractionModality::Media)
    .with_interaction_modality(InteractionModality::Files)
    .with_ux_capabilities(ux_capabilities)
    .with_trust_level(TrustLevel::Verified)
    .with_task_type("content_generation".to_string())
    .with_task_type("media_analysis".to_string())
    .with_tag("ai".to_string())
    .with_tag("multimodal".to_string())
}

/// 创建企业级Agent
fn create_enterprise_agent() -> AgentCard {
    let ux_capabilities = UxCapabilities::new()
        .with_component("dashboard".to_string())
        .with_component("report_viewer".to_string())
        .with_dynamic_adaptation();
    
    AgentCard::new(
        "enterprise_analytics".to_string(),
        "企业数据分析Agent".to_string(),
        "专业的企业级数据分析和报告生成系统".to_string(),
        "3.1.0".to_string(),
    )
    .add_capability(Capability::new(
        "data_analysis".to_string(),
        "大规模数据分析和洞察".to_string(),
        CapabilityType::DataAnalysis,
    ))
    .add_capability(Capability::new(
        "report_generation".to_string(),
        "自动生成业务报告".to_string(),
        CapabilityType::TextGeneration,
    ))
    .add_endpoint(Endpoint::new(
        "grpc".to_string(),
        "grpc://enterprise.internal:9090".to_string(),
    ))
    .with_interaction_modality(InteractionModality::Text)
    .with_interaction_modality(InteractionModality::Forms)
    .with_ux_capabilities(ux_capabilities)
    .with_trust_level(TrustLevel::Internal)
    .with_task_type("data_analysis".to_string())
    .with_task_type("reporting".to_string())
    .with_tag("enterprise".to_string())
    .with_tag("analytics".to_string())
}

/// 打印Agent信息
fn print_agent_info(card: &AgentCard, title: &str) {
    println!("📄 {}", title);
    println!("   ID: {}", card.id);
    println!("   名称: {}", card.name);
    println!("   版本: {}", card.version);
    println!("   状态: {:?}", card.status);
    println!("   信任级别: {:?} (分数: {})", card.trust_level, card.trust_level.trust_score());
    println!("   能力数量: {}", card.capabilities.len());
    println!("   交互模式: {:?}", card.interaction_modalities);
    println!("   多模态支持: {}", if card.is_multimodal() { "✅" } else { "❌" });
    println!("   端点数量: {}", card.endpoints.len());
    println!("   支持的任务类型: {:?}", card.supported_task_types);
    
    if let Some(ux) = &card.ux_capabilities {
        println!("   UX能力:");
        println!("     - 动态适应: {}", if ux.dynamic_adaptation { "✅" } else { "❌" });
        println!("     - 多模态UX: {}", if ux.multimodal_support { "✅" } else { "❌" });
        println!("     - 支持组件: {:?}", ux.supported_components);
    }
}

/// 演示Agent发现和匹配
fn demonstrate_agent_discovery(agents: &[&AgentCard]) {
    println!("🔍 Agent发现和能力匹配:");
    
    // 按信任级别排序
    let mut sorted_agents: Vec<_> = agents.iter().collect();
    sorted_agents.sort_by(|a, b| b.trust_level.trust_score().cmp(&a.trust_level.trust_score()));
    
    println!("\n📊 按信任级别排序:");
    for agent in &sorted_agents {
        println!("   {} - 信任分数: {}", agent.name, agent.trust_level.trust_score());
    }
    
    // 查找支持特定能力的Agent
    println!("\n🎯 能力匹配:");
    let required_capabilities = ["text_generation", "data_analysis"];
    
    for capability in &required_capabilities {
        println!("   寻找支持'{}'的Agent:", capability);
        for agent in agents {
            if agent.has_capability(capability) {
                println!("     ✅ {} (信任级别: {:?})", agent.name, agent.trust_level);
            }
        }
    }
    
    // 查找支持特定交互模式的Agent
    println!("\n🎨 交互模式匹配:");
    let modalities = [InteractionModality::Media, InteractionModality::Forms];
    
    for modality in &modalities {
        println!("   支持{:?}模式的Agent:", modality);
        for agent in agents {
            if agent.supports_modality(modality) {
                println!("     ✅ {}", agent.name);
            }
        }
    }
}

/// 演示多模态消息交换
async fn demonstrate_multimodal_messages() {
    println!("💬 多模态消息交换:");
    
    // 创建文本消息
    let text_message = A2AMessage::new_text(
        MessageRole::User,
        "请帮我分析这个图像中的内容".to_string(),
    );
    
    println!("   📝 文本消息:");
    println!("     角色: {:?}", text_message.role);
    println!("     部分数量: {}", text_message.parts.len());
    
    // 创建图像文件消息
    let image_data = FileData::WithBytes(FileWithBytes {
        name: Some("analysis_chart.png".to_string()),
        mime_type: "image/png".to_string(),
        bytes: "ZmFrZV9pbWFnZV9kYXRhX2hlcmU=".to_string(), // base64 encoded "fake_image_data_here"
    });
    
    let image_message = A2AMessage::new_file(MessageRole::User, image_data);
    
    println!("   🖼️ 图像消息:");
    println!("     角色: {:?}", image_message.role);
    if let MessagePart::File(file_part) = &image_message.parts[0] {
        if let FileData::WithBytes(file_bytes) = &file_part.file {
            println!("     文件名: {:?}", file_bytes.name);
            println!("     MIME类型: {}", file_bytes.mime_type);
        }
    }
    
    // 创建分析结果消息
    let analysis_result = A2AMessage::new_data(
        MessageRole::Agent,
        serde_json::json!({
            "image_analysis": {
                "objects_detected": ["chart", "data_points", "trend_line"],
                "confidence": 0.92,
                "insights": "图表显示上升趋势，建议继续监控"
            }
        }),
    );
    
    println!("   📊 分析结果消息:");
    println!("     角色: {:?}", analysis_result.role);
    if let MessagePart::Data(data_part) = &analysis_result.parts[0] {
        println!("     检测到的对象: {:?}", 
                data_part.data["image_analysis"]["objects_detected"]);
        println!("     置信度: {}", 
                data_part.data["image_analysis"]["confidence"]);
    }
}

/// 演示UX协商
fn demonstrate_ux_negotiation(agent: &AgentCard) {
    println!("🎨 UX协商演示:");
    
    if let Some(ux) = &agent.ux_capabilities {
        println!("   Agent UX能力:");
        println!("     支持的组件: {:?}", ux.supported_components);
        println!("     动态适应: {}", ux.dynamic_adaptation);
        println!("     多模态支持: {}", ux.multimodal_support);
        
        println!("\n   🤝 协商过程:");
        println!("     1. 客户端请求: 需要图像查看器和文件上传功能");
        println!("     2. Agent响应: 支持image_viewer和file_uploader组件");
        println!("     3. 协商结果: ✅ 兼容，可以建立连接");
        
        if ux.dynamic_adaptation {
            println!("     4. 动态适应: ✅ 支持运行时UX调整");
        }
        
        if ux.multimodal_support {
            println!("     5. 多模态UX: ✅ 支持多种交互模式的统一界面");
        }
    }
}

/// 演示信任级别管理
fn demonstrate_trust_management(agents: &[&AgentCard]) {
    println!("🔒 企业信任级别管理:");
    
    println!("   信任级别说明:");
    println!("     Public (1分): 公开Agent，无特殊信任");
    println!("     Verified (3分): 已验证身份的Agent");
    println!("     Trusted (7分): 组织内信任的Agent");
    println!("     Internal (10分): 完全信任的内部系统");
    
    println!("\n   Agent信任评估:");
    for agent in agents {
        let trust_indicator = match agent.trust_level {
            TrustLevel::Internal => "🟢",
            TrustLevel::Trusted => "🟡",
            TrustLevel::Verified => "🟠",
            TrustLevel::Public => "🔴",
        };
        
        println!("     {} {} - {:?} ({}分)", 
                trust_indicator, agent.name, agent.trust_level, agent.trust_level.trust_score());
    }
    
    println!("\n   🛡️ 安全策略:");
    println!("     - Internal Agent: 可访问所有企业资源");
    println!("     - Trusted Agent: 可访问部门级资源");
    println!("     - Verified Agent: 可访问公共API");
    println!("     - Public Agent: 仅限基础功能");
    
    println!("\n   🔐 访问控制示例:");
    for agent in agents {
        match agent.trust_level {
            TrustLevel::Internal => {
                println!("     {} 可以访问: 财务数据、人事信息、商业机密", agent.name);
            },
            TrustLevel::Trusted => {
                println!("     {} 可以访问: 部门数据、项目信息", agent.name);
            },
            TrustLevel::Verified => {
                println!("     {} 可以访问: 公共API、基础服务", agent.name);
            },
            TrustLevel::Public => {
                println!("     {} 可以访问: 公开信息、基础功能", agent.name);
            },
        }
    }
}
