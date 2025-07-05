//! A2A协议与gRPC消息转换器
//!
//! 提供A2A协议类型和gRPC protobuf类型之间的真实转换

use crate::error::{GrpcError, GrpcResult};
use crate::proto::{
    A2aMessageRequest, AgentInfo as ProtoAgentInfo,
    MessageType, AgentStatus as ProtoAgentStatus, TrustLevel as ProtoTrustLevel,
};
use agentx_a2a::{
    A2AMessage, MessageRole, MessagePart, TextPart,
    AgentCard, Capability, CapabilityType, Endpoint,
    AgentStatus, TrustLevel,
};
use prost_types::Timestamp;
use std::collections::HashMap;
use serde_json;

/// A2A消息转换器
///
/// 提供A2A协议类型和gRPC protobuf类型之间的真实转换
pub struct A2AConverter;

impl A2AConverter {
    /// 将A2A消息转换为gRPC请求
    pub fn a2a_to_grpc_request(message: &A2AMessage) -> GrpcResult<A2aMessageRequest> {
        let timestamp = Some(Timestamp {
            seconds: chrono::Utc::now().timestamp(),
            nanos: chrono::Utc::now().timestamp_subsec_nanos() as i32,
        });

        // 序列化消息内容为JSON
        let payload_json = serde_json::to_string(&message.parts)
            .map_err(|e| GrpcError::protocol_conversion(format!("序列化消息内容失败: {}", e)))?;

        let payload = Some(prost_types::Any {
            type_url: "type.googleapis.com/agentx.a2a.v1.MessageParts".to_string(),
            value: payload_json.into_bytes(),
        });

        Ok(A2aMessageRequest {
            message_id: message.message_id.clone(),
            from_agent: "".to_string(), // A2A消息没有from字段，使用空字符串
            to_agent: "".to_string(),   // A2A消息没有to字段，使用空字符串
            message_type: Self::message_role_to_grpc_type(&message.role),
            payload,
            metadata: message.metadata.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
            timestamp,
            ttl_seconds: 300, // 默认5分钟TTL
        })
    }

    /// 将gRPC响应转换为A2A消息
    pub fn grpc_response_to_a2a(response: A2aMessageRequest) -> GrpcResult<A2AMessage> {
        let role = Self::grpc_type_to_message_role(response.message_type)?;

        // 反序列化消息内容
        let parts = if let Some(payload) = response.payload {
            let payload_str = String::from_utf8(payload.value)
                .map_err(|e| GrpcError::protocol_conversion(format!("解析payload失败: {}", e)))?;

            serde_json::from_str::<Vec<MessagePart>>(&payload_str)
                .map_err(|e| GrpcError::protocol_conversion(format!("反序列化消息内容失败: {}", e)))?
        } else {
            vec![MessagePart::Text(TextPart {
                text: "空消息".to_string(),
                metadata: HashMap::new(),
            })]
        };

        Ok(A2AMessage {
            message_id: response.message_id,
            task_id: None,
            context_id: None,
            role,
            parts,
            metadata: response.metadata.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
        })
    }

    /// 将Agent Card转换为gRPC Agent Info
    pub fn agent_card_to_grpc_info(card: &AgentCard) -> GrpcResult<ProtoAgentInfo> {
        let created_at = Some(Timestamp {
            seconds: card.created_at.timestamp(),
            nanos: card.created_at.timestamp_subsec_nanos() as i32,
        });

        let updated_at = Some(Timestamp {
            seconds: card.updated_at.timestamp(),
            nanos: card.updated_at.timestamp_subsec_nanos() as i32,
        });

        Ok(ProtoAgentInfo {
            id: card.id.clone(),
            name: card.name.clone(),
            description: card.description.clone(),
            framework: "agentx".to_string(), // 默认框架
            version: card.version.clone(),
            status: Self::agent_status_to_grpc(card.status),
            trust_level: Self::trust_level_to_grpc(card.trust_level),
            tags: card.tags.clone(),
            metadata: card.metadata.iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
            created_at,
            updated_at,
        })
    }

    /// 将gRPC Agent Info转换为Agent Card
    pub fn grpc_info_to_agent_card(info: ProtoAgentInfo) -> GrpcResult<AgentCard> {
        let created_at = info.created_at
            .map(|ts| chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32))
            .flatten()
            .unwrap_or_else(chrono::Utc::now);

        let updated_at = info.updated_at
            .map(|ts| chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32))
            .flatten()
            .unwrap_or_else(chrono::Utc::now);

        Ok(AgentCard {
            id: info.id,
            name: info.name,
            description: info.description,
            version: info.version,
            capabilities: Vec::new(), // 需要单独获取
            endpoints: Vec::new(), // 需要单独获取
            metadata: info.metadata.into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect(),
            created_at,
            updated_at,
            expires_at: None,
            status: Self::grpc_to_agent_status(info.status)?,
            supported_versions: vec!["0.2.5".to_string()],
            tags: info.tags,
            interaction_modalities: Vec::new(),
            ux_capabilities: None,
            trust_level: Self::grpc_to_trust_level(info.trust_level)?,
            supported_task_types: Vec::new(),
        })
    }

    // 私有转换方法

    fn message_role_to_grpc_type(role: &MessageRole) -> i32 {
        match role {
            MessageRole::User => MessageType::Request as i32,
            MessageRole::Agent => MessageType::Response as i32,
        }
    }

    fn grpc_type_to_message_role(message_type: i32) -> GrpcResult<MessageRole> {
        match MessageType::try_from(message_type) {
            Ok(MessageType::Request) => Ok(MessageRole::User),
            Ok(MessageType::Response) => Ok(MessageRole::Agent),
            _ => Ok(MessageRole::Agent), // 默认为Agent
        }
    }

    fn agent_status_to_grpc(status: AgentStatus) -> i32 {
        match status {
            AgentStatus::Online => ProtoAgentStatus::Online as i32,
            AgentStatus::Offline => ProtoAgentStatus::Offline as i32,
            AgentStatus::Busy => ProtoAgentStatus::Busy as i32,
            AgentStatus::Maintenance => ProtoAgentStatus::Error as i32,
            AgentStatus::Unknown => ProtoAgentStatus::Unspecified as i32,
        }
    }

    fn grpc_to_agent_status(status: i32) -> GrpcResult<AgentStatus> {
        match ProtoAgentStatus::try_from(status) {
            Ok(ProtoAgentStatus::Online) => Ok(AgentStatus::Online),
            Ok(ProtoAgentStatus::Offline) => Ok(AgentStatus::Offline),
            Ok(ProtoAgentStatus::Busy) => Ok(AgentStatus::Busy),
            Ok(ProtoAgentStatus::Error) => Ok(AgentStatus::Maintenance),
            _ => Ok(AgentStatus::Unknown),
        }
    }

    fn trust_level_to_grpc(trust: TrustLevel) -> i32 {
        match trust {
            TrustLevel::Public => ProtoTrustLevel::Public as i32,
            TrustLevel::Verified => ProtoTrustLevel::Verified as i32,
            TrustLevel::Trusted => ProtoTrustLevel::Trusted as i32,
            TrustLevel::Internal => ProtoTrustLevel::Internal as i32,
        }
    }

    fn grpc_to_trust_level(trust: i32) -> GrpcResult<TrustLevel> {
        match ProtoTrustLevel::try_from(trust) {
            Ok(ProtoTrustLevel::Public) => Ok(TrustLevel::Public),
            Ok(ProtoTrustLevel::Verified) => Ok(TrustLevel::Verified),
            Ok(ProtoTrustLevel::Trusted) => Ok(TrustLevel::Trusted),
            Ok(ProtoTrustLevel::Internal) => Ok(TrustLevel::Internal),
            _ => Ok(TrustLevel::Public),
        }
    }

    /// 将A2A消息转换为JSON格式（向后兼容）
    pub fn message_to_json(message: &A2AMessage) -> GrpcResult<serde_json::Value> {
        let json = serde_json::json!({
            "message_id": message.message_id,
            "task_id": message.task_id,
            "context_id": message.context_id,
            "role": Self::role_to_string(&message.role),
            "parts": message.parts.iter()
                .map(Self::part_to_json)
                .collect::<GrpcResult<Vec<_>>>()?,
            "metadata": message.metadata,
        });
        
        Ok(json)
    }
    
    /// 从JSON格式转换为A2A消息（模拟gRPC反序列化）
    pub fn message_from_json(json: &serde_json::Value) -> GrpcResult<A2AMessage> {
        let message_id = json["message_id"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少message_id字段"))?
            .to_string();
        
        let task_id = json["task_id"].as_str()
            .map(|s| s.to_string());

        let context_id = json["context_id"].as_str()
            .map(|s| s.to_string());
        
        let role = Self::role_from_string(
            json["role"].as_str()
                .ok_or_else(|| GrpcError::protocol_conversion("缺少role字段"))?
        )?;
        
        let parts = json["parts"].as_array()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少parts字段"))?
            .iter()
            .map(Self::part_from_json)
            .collect::<GrpcResult<Vec<_>>>()?;
        
        let metadata = json["metadata"].as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(A2AMessage {
            message_id,
            task_id,
            context_id,
            role,
            parts,
            metadata,
        })
    }
    
    /// 将Agent Card转换为JSON格式
    pub fn agent_card_to_json(card: &AgentCard) -> GrpcResult<serde_json::Value> {
        let json = serde_json::json!({
            "id": card.id,
            "name": card.name,
            "description": card.description,
            "version": card.version,
            "capabilities": card.capabilities.iter()
                .map(Self::capability_to_json)
                .collect::<GrpcResult<Vec<_>>>()?,
            "endpoints": card.endpoints.iter()
                .map(Self::endpoint_to_json)
                .collect::<GrpcResult<Vec<_>>>()?,
            "metadata": card.metadata,
            "created_at": card.created_at.to_rfc3339(),
            "updated_at": card.updated_at.to_rfc3339(),
            "status": Self::agent_status_to_string(&card.status),
            "trust_level": Self::trust_level_to_string(&card.trust_level),
            "interaction_modalities": card.interaction_modalities.iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>(),
            "supported_task_types": card.supported_task_types,
            "tags": card.tags,
        });
        
        Ok(json)
    }
    
    /// 从JSON格式转换为Agent Card
    pub fn agent_card_from_json(json: &serde_json::Value) -> GrpcResult<AgentCard> {
        let id = json["id"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少id字段"))?
            .to_string();
        
        let name = json["name"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少name字段"))?
            .to_string();
        
        let description = json["description"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少description字段"))?
            .to_string();
        
        let version = json["version"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少version字段"))?
            .to_string();
        
        let capabilities = json["capabilities"].as_array()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少capabilities字段"))?
            .iter()
            .map(Self::capability_from_json)
            .collect::<GrpcResult<Vec<_>>>()?;
        
        let endpoints = json["endpoints"].as_array()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少endpoints字段"))?
            .iter()
            .map(Self::endpoint_from_json)
            .collect::<GrpcResult<Vec<_>>>()?;
        
        let metadata = json["metadata"].as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        let created_at = json["created_at"].as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);
        
        let updated_at = json["updated_at"].as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);
        
        let status = json["status"].as_str()
            .map(Self::agent_status_from_string)
            .transpose()?
            .unwrap_or(AgentStatus::Online);
        
        let trust_level = json["trust_level"].as_str()
            .map(Self::trust_level_from_string)
            .transpose()?
            .unwrap_or(TrustLevel::Public);
        
        let card = AgentCard {
            id,
            name,
            description,
            version,
            capabilities,
            endpoints,
            metadata,
            created_at,
            updated_at,
            expires_at: None,
            status,
            supported_versions: vec!["0.2.5".to_string()],
            tags: json["tags"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
            interaction_modalities: Vec::new(), // 简化实现
            ux_capabilities: None,
            trust_level,
            supported_task_types: json["supported_task_types"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
        };
        
        Ok(card)
    }
    
    // 私有辅助方法
    
    fn role_to_string(role: &MessageRole) -> &'static str {
        match role {
            MessageRole::User => "user",
            MessageRole::Agent => "agent",
        }
    }

    fn role_from_string(role_str: &str) -> GrpcResult<MessageRole> {
        match role_str {
            "user" => Ok(MessageRole::User),
            "agent" => Ok(MessageRole::Agent),
            _ => Err(GrpcError::protocol_conversion(format!("无效的消息角色: {}", role_str))),
        }
    }
    
    fn part_to_json(part: &MessagePart) -> GrpcResult<serde_json::Value> {
        match part {
            MessagePart::Text(text_part) => {
                Ok(serde_json::json!({
                    "type": "text",
                    "text": text_part.text,
                    "metadata": text_part.metadata,
                }))
            },
            MessagePart::File(file_part) => {
                let (name, mime_type) = match &file_part.file {
                    agentx_a2a::FileData::WithBytes(bytes) => (
                        bytes.name.clone().unwrap_or_default(),
                        bytes.mime_type.clone()
                    ),
                    agentx_a2a::FileData::WithUri(uri) => (
                        uri.name.clone().unwrap_or_default(),
                        uri.mime_type.clone()
                    ),
                };

                Ok(serde_json::json!({
                    "type": "file",
                    "file": {
                        "name": name,
                        "mime_type": mime_type,
                        "data": "base64_encoded_data", // 简化实现
                    },
                    "metadata": file_part.metadata,
                }))
            },
            MessagePart::Data(data_part) => {
                Ok(serde_json::json!({
                    "type": "data",
                    "data": data_part.data,
                    "metadata": data_part.metadata,
                }))
            },
        }
    }
    
    fn part_from_json(json: &serde_json::Value) -> GrpcResult<MessagePart> {
        let part_type = json["type"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少type字段"))?;
        
        match part_type {
            "text" => {
                let text = json["text"].as_str()
                    .ok_or_else(|| GrpcError::protocol_conversion("缺少text字段"))?
                    .to_string();
                
                let metadata = json["metadata"].as_object()
                    .unwrap_or(&serde_json::Map::new())
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                
                Ok(MessagePart::Text(TextPart { text, metadata }))
            },
            _ => Err(GrpcError::protocol_conversion(format!("不支持的消息部分类型: {}", part_type))),
        }
    }
    
    fn capability_to_json(cap: &Capability) -> GrpcResult<serde_json::Value> {
        Ok(serde_json::json!({
            "name": cap.name,
            "description": cap.description,
            "type": Self::capability_type_to_string(&cap.capability_type),
            "available": cap.available,
            "input_schema": cap.input_schema,
            "output_schema": cap.output_schema,
            "metadata": cap.metadata,
        }))
    }
    
    fn capability_from_json(json: &serde_json::Value) -> GrpcResult<Capability> {
        let name = json["name"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少name字段"))?
            .to_string();
        
        let description = json["description"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少description字段"))?
            .to_string();
        
        let capability_type = json["type"].as_str()
            .map(Self::capability_type_from_string)
            .transpose()?
            .unwrap_or(CapabilityType::Custom("unknown".to_string()));
        
        Ok(Capability {
            name,
            description,
            capability_type,
            available: json["available"].as_bool().unwrap_or(true),
            input_schema: json["input_schema"].clone().into(),
            output_schema: json["output_schema"].clone().into(),
            metadata: json["metadata"].as_object()
                .unwrap_or(&serde_json::Map::new())
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            cost: None, // 简化实现
        })
    }
    
    fn endpoint_to_json(endpoint: &Endpoint) -> GrpcResult<serde_json::Value> {
        Ok(serde_json::json!({
            "type": endpoint.endpoint_type,
            "url": endpoint.url,
            "protocols": endpoint.protocols,
            "metadata": endpoint.metadata,
        }))
    }
    
    fn endpoint_from_json(json: &serde_json::Value) -> GrpcResult<Endpoint> {
        let endpoint_type = json["type"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少type字段"))?
            .to_string();
        
        let url = json["url"].as_str()
            .ok_or_else(|| GrpcError::protocol_conversion("缺少url字段"))?
            .to_string();
        
        Ok(Endpoint {
            endpoint_type,
            url,
            protocols: json["protocols"].as_array()
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default(),
            auth: None, // 简化实现
            metadata: json["metadata"].as_object()
                .unwrap_or(&serde_json::Map::new())
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        })
    }
    
    fn capability_type_to_string(cap_type: &CapabilityType) -> &'static str {
        match cap_type {
            CapabilityType::TextGeneration => "text_generation",
            CapabilityType::ImageProcessing => "image_processing",
            CapabilityType::AudioProcessing => "audio_processing",
            CapabilityType::VideoProcessing => "video_processing",
            CapabilityType::DataAnalysis => "data_analysis",
            CapabilityType::CodeExecution => "code_execution",
            CapabilityType::ToolExecution => "tool_execution",
            CapabilityType::WorkflowOrchestration => "workflow_orchestration",
            CapabilityType::KnowledgeRetrieval => "knowledge_retrieval",
            CapabilityType::Custom(_) => "custom",
        }
    }
    
    fn capability_type_from_string(type_str: &str) -> GrpcResult<CapabilityType> {
        match type_str {
            "text_generation" => Ok(CapabilityType::TextGeneration),
            "image_processing" => Ok(CapabilityType::ImageProcessing),
            "audio_processing" => Ok(CapabilityType::AudioProcessing),
            "video_processing" => Ok(CapabilityType::VideoProcessing),
            "data_analysis" => Ok(CapabilityType::DataAnalysis),
            "code_execution" => Ok(CapabilityType::CodeExecution),
            "custom" => Ok(CapabilityType::Custom("custom".to_string())),
            _ => Ok(CapabilityType::Custom(type_str.to_string())),
        }
    }
    
    fn agent_status_to_string(status: &AgentStatus) -> &'static str {
        match status {
            AgentStatus::Online => "online",
            AgentStatus::Offline => "offline",
            AgentStatus::Busy => "busy",
            AgentStatus::Maintenance => "maintenance",
            AgentStatus::Unknown => "unknown",
        }
    }
    
    fn agent_status_from_string(status_str: &str) -> GrpcResult<AgentStatus> {
        match status_str {
            "online" => Ok(AgentStatus::Online),
            "offline" => Ok(AgentStatus::Offline),
            "busy" => Ok(AgentStatus::Busy),
            "maintenance" => Ok(AgentStatus::Maintenance),
            _ => Err(GrpcError::protocol_conversion(format!("无效的Agent状态: {}", status_str))),
        }
    }
    
    fn trust_level_to_string(trust: &TrustLevel) -> &'static str {
        match trust {
            TrustLevel::Public => "public",
            TrustLevel::Verified => "verified",
            TrustLevel::Trusted => "trusted",
            TrustLevel::Internal => "internal",
        }
    }
    
    fn trust_level_from_string(trust_str: &str) -> GrpcResult<TrustLevel> {
        match trust_str {
            "public" => Ok(TrustLevel::Public),
            "verified" => Ok(TrustLevel::Verified),
            "trusted" => Ok(TrustLevel::Trusted),
            "internal" => Ok(TrustLevel::Internal),
            _ => Err(GrpcError::protocol_conversion(format!("无效的信任级别: {}", trust_str))),
        }
    }
}
