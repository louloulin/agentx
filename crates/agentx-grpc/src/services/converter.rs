//! A2A协议与gRPC消息之间的转换器
//! 
//! 提供A2A协议类型和gRPC protobuf类型之间的双向转换

use crate::error::{GrpcError, GrpcResult};
use agentx_a2a::{
    A2AMessage, MessageRole, MessagePart, TextPart, FilePart, DataPart,
    FileData, FileWithBytes, FileWithUri, AgentCard, Capability, CapabilityType,
    Endpoint, AuthInfo, AgentStatus, InteractionModality, TrustLevel,
    A2ATask, TaskState, TaskStatus,
};
use crate::generated::agentx::a2a::v1 as grpc_a2a;
use crate::generated::agentx::registry::v1 as grpc_registry;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// A2A消息转换器
pub struct A2AConverter;

impl A2AConverter {
    /// 将A2A消息转换为gRPC消息
    pub fn to_grpc_message(message: &A2AMessage) -> GrpcResult<grpc_a2a::A2AMessage> {
        Ok(grpc_a2a::A2AMessage {
            message_id: message.message_id.clone(),
            conversation_id: message.conversation_id.clone().unwrap_or_default(),
            role: Self::role_to_grpc(&message.role) as i32,
            parts: message.parts.iter()
                .map(Self::part_to_grpc)
                .collect::<GrpcResult<Vec<_>>>()?,
            metadata: Self::metadata_to_grpc(&message.metadata),
            timestamp: message.timestamp,
        })
    }
    
    /// 将gRPC消息转换为A2A消息
    pub fn from_grpc_message(grpc_msg: &grpc_a2a::A2AMessage) -> GrpcResult<A2AMessage> {
        let role = Self::role_from_grpc(grpc_msg.role)?;
        let parts = grpc_msg.parts.iter()
            .map(Self::part_from_grpc)
            .collect::<GrpcResult<Vec<_>>>()?;
        
        Ok(A2AMessage {
            message_id: grpc_msg.message_id.clone(),
            conversation_id: if grpc_msg.conversation_id.is_empty() {
                None
            } else {
                Some(grpc_msg.conversation_id.clone())
            },
            role,
            parts,
            metadata: Self::metadata_from_grpc(&grpc_msg.metadata),
            timestamp: grpc_msg.timestamp,
        })
    }
    
    /// 将A2A Agent Card转换为gRPC Agent Card
    pub fn agent_card_to_grpc(card: &AgentCard) -> GrpcResult<grpc_registry::AgentCard> {
        Ok(grpc_registry::AgentCard {
            id: card.id.clone(),
            name: card.name.clone(),
            description: card.description.clone(),
            version: card.version.clone(),
            capabilities: card.capabilities.iter()
                .map(Self::capability_to_grpc)
                .collect::<GrpcResult<Vec<_>>>()?,
            endpoints: card.endpoints.iter()
                .map(Self::endpoint_to_grpc)
                .collect::<GrpcResult<Vec<_>>>()?,
            metadata: Self::metadata_to_grpc(&card.metadata),
            created_at: card.created_at,
            updated_at: card.updated_at,
            status: Self::agent_status_to_grpc(&card.status) as i32,
            trust_level: Self::trust_level_to_grpc(&card.trust_level) as i32,
        })
    }
    
    /// 将gRPC Agent Card转换为A2A Agent Card
    pub fn agent_card_from_grpc(grpc_card: &grpc_registry::AgentCard) -> GrpcResult<AgentCard> {
        let capabilities = grpc_card.capabilities.iter()
            .map(Self::capability_from_grpc)
            .collect::<GrpcResult<Vec<_>>>()?;
        
        let endpoints = grpc_card.endpoints.iter()
            .map(Self::endpoint_from_grpc)
            .collect::<GrpcResult<Vec<_>>>()?;
        
        Ok(AgentCard {
            id: grpc_card.id.clone(),
            name: grpc_card.name.clone(),
            description: grpc_card.description.clone(),
            version: grpc_card.version.clone(),
            capabilities,
            endpoints,
            metadata: Self::metadata_from_grpc(&grpc_card.metadata),
            created_at: grpc_card.created_at,
            updated_at: grpc_card.updated_at,
            expires_at: None, // TODO: 添加到protobuf定义
            status: Self::agent_status_from_grpc(grpc_card.status)?,
            supported_versions: vec!["0.2.5".to_string()], // TODO: 添加到protobuf定义
            tags: Vec::new(), // TODO: 添加到protobuf定义
            interaction_modalities: Vec::new(), // TODO: 添加到protobuf定义
            ux_capabilities: None, // TODO: 添加到protobuf定义
            trust_level: Self::trust_level_from_grpc(grpc_card.trust_level)?,
            supported_task_types: Vec::new(), // TODO: 添加到protobuf定义
        })
    }
    
    // 私有辅助方法
    
    fn role_to_grpc(role: &MessageRole) -> grpc_a2a::MessageRole {
        match role {
            MessageRole::User => grpc_a2a::MessageRole::MessageRoleUser,
            MessageRole::Agent => grpc_a2a::MessageRole::MessageRoleAgent,
            MessageRole::System => grpc_a2a::MessageRole::MessageRoleSystem,
        }
    }
    
    fn role_from_grpc(role: i32) -> GrpcResult<MessageRole> {
        match role {
            1 => Ok(MessageRole::User),
            2 => Ok(MessageRole::Agent),
            3 => Ok(MessageRole::System),
            _ => Err(GrpcError::protocol_conversion(format!("无效的消息角色: {}", role))),
        }
    }
    
    fn part_to_grpc(part: &MessagePart) -> GrpcResult<grpc_a2a::MessagePart> {
        let content = match part {
            MessagePart::Text(text_part) => {
                grpc_a2a::message_part::Content::Text(grpc_a2a::TextPart {
                    text: text_part.text.clone(),
                    metadata: Self::metadata_to_grpc(&text_part.metadata),
                })
            },
            MessagePart::File(file_part) => {
                grpc_a2a::message_part::Content::File(grpc_a2a::FilePart {
                    file: Some(Self::file_data_to_grpc(&file_part.file)?),
                    metadata: Self::metadata_to_grpc(&file_part.metadata),
                })
            },
            MessagePart::Data(data_part) => {
                grpc_a2a::message_part::Content::Data(grpc_a2a::DataPart {
                    data: Some(Self::json_to_struct(&data_part.data)?),
                    metadata: Self::metadata_to_grpc(&data_part.metadata),
                })
            },
        };
        
        Ok(grpc_a2a::MessagePart {
            content: Some(content),
        })
    }
    
    fn part_from_grpc(grpc_part: &grpc_a2a::MessagePart) -> GrpcResult<MessagePart> {
        match &grpc_part.content {
            Some(grpc_a2a::message_part::Content::Text(text)) => {
                Ok(MessagePart::Text(TextPart {
                    text: text.text.clone(),
                    metadata: Self::metadata_from_grpc(&text.metadata),
                }))
            },
            Some(grpc_a2a::message_part::Content::File(file)) => {
                let file_data = file.file.as_ref()
                    .ok_or_else(|| GrpcError::protocol_conversion("缺少文件数据"))?;
                
                Ok(MessagePart::File(FilePart {
                    file: Self::file_data_from_grpc(file_data)?,
                    metadata: Self::metadata_from_grpc(&file.metadata),
                }))
            },
            Some(grpc_a2a::message_part::Content::Data(data)) => {
                let data_value = data.data.as_ref()
                    .ok_or_else(|| GrpcError::protocol_conversion("缺少数据内容"))?;
                
                Ok(MessagePart::Data(DataPart {
                    data: Self::struct_to_json(data_value)?,
                    metadata: Self::metadata_from_grpc(&data.metadata),
                }))
            },
            _ => Err(GrpcError::protocol_conversion("未知的消息部分类型")),
        }
    }
    
    fn file_data_to_grpc(file_data: &FileData) -> GrpcResult<grpc_a2a::FileData> {
        let data = match file_data {
            FileData::WithBytes(file_bytes) => {
                grpc_a2a::file_data::Data::WithBytes(grpc_a2a::FileWithBytes {
                    name: file_bytes.name.clone(),
                    mime_type: file_bytes.mime_type.clone(),
                    bytes: file_bytes.bytes.clone(),
                })
            },
            FileData::WithUri(file_uri) => {
                grpc_a2a::file_data::Data::WithUri(grpc_a2a::FileWithUri {
                    name: file_uri.name.clone(),
                    mime_type: file_uri.mime_type.clone(),
                    uri: file_uri.uri.clone(),
                })
            },
        };
        
        Ok(grpc_a2a::FileData {
            data: Some(data),
        })
    }
    
    fn file_data_from_grpc(grpc_file: &grpc_a2a::FileData) -> GrpcResult<FileData> {
        match &grpc_file.data {
            Some(grpc_a2a::file_data::Data::WithBytes(bytes)) => {
                Ok(FileData::WithBytes(FileWithBytes {
                    name: bytes.name.clone(),
                    mime_type: bytes.mime_type.clone(),
                    bytes: bytes.bytes.clone(),
                }))
            },
            Some(grpc_a2a::file_data::Data::WithUri(uri)) => {
                Ok(FileData::WithUri(FileWithUri {
                    name: uri.name.clone(),
                    mime_type: uri.mime_type.clone(),
                    uri: uri.uri.clone(),
                }))
            },
            _ => Err(GrpcError::protocol_conversion("无效的文件数据类型")),
        }
    }
    
    fn capability_to_grpc(cap: &Capability) -> GrpcResult<grpc_registry::Capability> {
        Ok(grpc_registry::Capability {
            name: cap.name.clone(),
            description: cap.description.clone(),
            capability_type: Self::capability_type_to_grpc(&cap.capability_type) as i32,
            available: cap.available,
            input_schema: cap.input_schema.as_ref().map(Self::json_to_struct).transpose()?,
            output_schema: cap.output_schema.as_ref().map(Self::json_to_struct).transpose()?,
            metadata: Self::metadata_to_grpc(&cap.metadata),
        })
    }
    
    fn capability_from_grpc(grpc_cap: &grpc_registry::Capability) -> GrpcResult<Capability> {
        Ok(Capability {
            name: grpc_cap.name.clone(),
            description: grpc_cap.description.clone(),
            capability_type: Self::capability_type_from_grpc(grpc_cap.capability_type)?,
            available: grpc_cap.available,
            input_schema: grpc_cap.input_schema.as_ref()
                .map(Self::struct_to_json).transpose()?,
            output_schema: grpc_cap.output_schema.as_ref()
                .map(Self::struct_to_json).transpose()?,
            metadata: Self::metadata_from_grpc(&grpc_cap.metadata),
        })
    }
    
    fn endpoint_to_grpc(endpoint: &Endpoint) -> GrpcResult<grpc_registry::Endpoint> {
        Ok(grpc_registry::Endpoint {
            endpoint_type: endpoint.endpoint_type.clone(),
            url: endpoint.url.clone(),
            protocol: endpoint.protocol.clone(),
            auth: endpoint.auth.as_ref().map(Self::auth_info_to_grpc).transpose()?,
            metadata: Self::metadata_to_grpc(&endpoint.metadata),
        })
    }
    
    fn endpoint_from_grpc(grpc_endpoint: &grpc_registry::Endpoint) -> GrpcResult<Endpoint> {
        Ok(Endpoint {
            endpoint_type: grpc_endpoint.endpoint_type.clone(),
            url: grpc_endpoint.url.clone(),
            protocol: grpc_endpoint.protocol.clone(),
            auth: grpc_endpoint.auth.as_ref()
                .map(Self::auth_info_from_grpc).transpose()?,
            metadata: Self::metadata_from_grpc(&grpc_endpoint.metadata),
        })
    }
    
    fn auth_info_to_grpc(auth: &AuthInfo) -> GrpcResult<grpc_registry::AuthInfo> {
        Ok(grpc_registry::AuthInfo {
            auth_type: auth.auth_type.clone(),
            parameters: Self::metadata_to_grpc(&auth.parameters),
        })
    }
    
    fn auth_info_from_grpc(grpc_auth: &grpc_registry::AuthInfo) -> GrpcResult<AuthInfo> {
        Ok(AuthInfo {
            auth_type: grpc_auth.auth_type.clone(),
            parameters: Self::metadata_from_grpc(&grpc_auth.parameters),
        })
    }
    
    // 枚举转换方法
    fn capability_type_to_grpc(cap_type: &CapabilityType) -> grpc_registry::CapabilityType {
        match cap_type {
            CapabilityType::TextGeneration => grpc_registry::CapabilityType::CapabilityTypeTextGeneration,
            CapabilityType::ImageProcessing => grpc_registry::CapabilityType::CapabilityTypeImageProcessing,
            CapabilityType::AudioProcessing => grpc_registry::CapabilityType::CapabilityTypeAudioProcessing,
            CapabilityType::VideoProcessing => grpc_registry::CapabilityType::CapabilityTypeVideoProcessing,
            CapabilityType::DataAnalysis => grpc_registry::CapabilityType::CapabilityTypeDataAnalysis,
            CapabilityType::CodeExecution => grpc_registry::CapabilityType::CapabilityTypeCodeExecution,
            CapabilityType::FileProcessing => grpc_registry::CapabilityType::CapabilityTypeFileProcessing,
            CapabilityType::WebSearch => grpc_registry::CapabilityType::CapabilityTypeWebSearch,
            CapabilityType::Custom(_) => grpc_registry::CapabilityType::CapabilityTypeCustom,
        }
    }
    
    fn capability_type_from_grpc(cap_type: i32) -> GrpcResult<CapabilityType> {
        match cap_type {
            1 => Ok(CapabilityType::TextGeneration),
            2 => Ok(CapabilityType::ImageProcessing),
            3 => Ok(CapabilityType::AudioProcessing),
            4 => Ok(CapabilityType::VideoProcessing),
            5 => Ok(CapabilityType::DataAnalysis),
            6 => Ok(CapabilityType::CodeExecution),
            7 => Ok(CapabilityType::FileProcessing),
            8 => Ok(CapabilityType::WebSearch),
            9 => Ok(CapabilityType::Custom("custom".to_string())),
            _ => Err(GrpcError::protocol_conversion(format!("无效的能力类型: {}", cap_type))),
        }
    }
    
    fn agent_status_to_grpc(status: &AgentStatus) -> grpc_registry::AgentStatus {
        match status {
            AgentStatus::Online => grpc_registry::AgentStatus::AgentStatusOnline,
            AgentStatus::Offline => grpc_registry::AgentStatus::AgentStatusOffline,
            AgentStatus::Busy => grpc_registry::AgentStatus::AgentStatusBusy,
            AgentStatus::Maintenance => grpc_registry::AgentStatus::AgentStatusMaintenance,
            AgentStatus::Error => grpc_registry::AgentStatus::AgentStatusError,
        }
    }
    
    fn agent_status_from_grpc(status: i32) -> GrpcResult<AgentStatus> {
        match status {
            1 => Ok(AgentStatus::Online),
            2 => Ok(AgentStatus::Offline),
            3 => Ok(AgentStatus::Busy),
            4 => Ok(AgentStatus::Maintenance),
            5 => Ok(AgentStatus::Error),
            _ => Err(GrpcError::protocol_conversion(format!("无效的Agent状态: {}", status))),
        }
    }
    
    fn trust_level_to_grpc(trust: &TrustLevel) -> grpc_registry::TrustLevel {
        match trust {
            TrustLevel::Public => grpc_registry::TrustLevel::TrustLevelPublic,
            TrustLevel::Verified => grpc_registry::TrustLevel::TrustLevelVerified,
            TrustLevel::Trusted => grpc_registry::TrustLevel::TrustLevelTrusted,
            TrustLevel::Internal => grpc_registry::TrustLevel::TrustLevelInternal,
        }
    }
    
    fn trust_level_from_grpc(trust: i32) -> GrpcResult<TrustLevel> {
        match trust {
            1 => Ok(TrustLevel::Public),
            2 => Ok(TrustLevel::Verified),
            3 => Ok(TrustLevel::Trusted),
            4 => Ok(TrustLevel::Internal),
            _ => Err(GrpcError::protocol_conversion(format!("无效的信任级别: {}", trust))),
        }
    }
    
    // 元数据转换辅助方法
    fn metadata_to_grpc(metadata: &HashMap<String, serde_json::Value>) -> prost_types::Struct {
        // 简化实现，实际应该正确转换
        prost_types::Struct {
            fields: HashMap::new(),
        }
    }
    
    fn metadata_from_grpc(grpc_metadata: &prost_types::Struct) -> HashMap<String, serde_json::Value> {
        // 简化实现，实际应该正确转换
        HashMap::new()
    }
    
    fn json_to_struct(value: &serde_json::Value) -> GrpcResult<prost_types::Struct> {
        // 简化实现，实际应该正确转换JSON到protobuf Struct
        Ok(prost_types::Struct {
            fields: HashMap::new(),
        })
    }
    
    fn struct_to_json(grpc_struct: &prost_types::Struct) -> GrpcResult<serde_json::Value> {
        // 简化实现，实际应该正确转换protobuf Struct到JSON
        Ok(serde_json::Value::Null)
    }
}
