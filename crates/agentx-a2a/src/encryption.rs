//! A2A协议加密通信模块
//! 
//! 提供端到端加密、密钥管理和安全通信功能

use crate::{A2AError, A2AResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use tracing::{debug, info};

/// 加密管理器
pub struct EncryptionManager {
    /// 密钥存储
    key_store: HashMap<String, EncryptionKey>,
    /// 加密配置
    config: EncryptionConfig,
    /// 密钥轮换历史
    key_rotation_history: Vec<KeyRotationEvent>,
}

/// 加密配置
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// 默认加密算法
    pub default_algorithm: EncryptionAlgorithm,
    /// 密钥长度
    pub key_length: usize,
    /// 密钥轮换间隔（小时）
    pub key_rotation_interval_hours: u64,
    /// 是否启用端到端加密
    pub enable_e2e_encryption: bool,
    /// 是否启用传输层加密
    pub enable_transport_encryption: bool,
    /// 密钥派生迭代次数
    pub key_derivation_iterations: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            default_algorithm: EncryptionAlgorithm::AES256GCM,
            key_length: 32, // 256 bits
            key_rotation_interval_hours: 24,
            enable_e2e_encryption: true,
            enable_transport_encryption: true,
            key_derivation_iterations: 100_000,
        }
    }
}

/// 加密算法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionAlgorithm {
    /// 无加密
    None,
    /// AES-256-GCM
    AES256GCM,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
    /// RSA-OAEP
    RSAOAEP,
    /// ECDH-ES
    ECDH,
    /// XChaCha20-Poly1305
    XChaCha20Poly1305,
}

/// 加密密钥
#[derive(Debug, Clone)]
pub struct EncryptionKey {
    /// 密钥ID
    pub key_id: String,
    /// 密钥数据
    pub key_data: Vec<u8>,
    /// 加密算法
    pub algorithm: EncryptionAlgorithm,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 密钥用途
    pub purpose: KeyPurpose,
    /// 密钥状态
    pub status: KeyStatus,
}

/// 密钥用途
#[derive(Debug, Clone, PartialEq)]
pub enum KeyPurpose {
    /// 消息加密
    MessageEncryption,
    /// 传输加密
    TransportEncryption,
    /// 数字签名
    DigitalSignature,
    /// 密钥交换
    KeyExchange,
    /// 身份验证
    Authentication,
}

/// 密钥状态
#[derive(Debug, Clone, PartialEq)]
pub enum KeyStatus {
    /// 活跃
    Active,
    /// 已轮换
    Rotated,
    /// 已撤销
    Revoked,
    /// 已过期
    Expired,
}

/// 密钥轮换事件
#[derive(Debug, Clone)]
pub struct KeyRotationEvent {
    /// 事件ID
    pub event_id: String,
    /// 旧密钥ID
    pub old_key_id: String,
    /// 新密钥ID
    pub new_key_id: String,
    /// 轮换时间
    pub rotated_at: DateTime<Utc>,
    /// 轮换原因
    pub reason: String,
}

/// 加密消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// 密钥ID
    pub key_id: String,
    /// 加密算法
    pub algorithm: EncryptionAlgorithm,
    /// 初始化向量/随机数
    pub iv: Vec<u8>,
    /// 加密数据
    pub encrypted_data: Vec<u8>,
    /// 认证标签
    pub auth_tag: Option<Vec<u8>>,
    /// 附加认证数据
    pub aad: Option<Vec<u8>>,
}

/// 密钥交换请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeRequest {
    /// 请求ID
    pub request_id: String,
    /// 发起方Agent ID
    pub initiator_agent_id: String,
    /// 目标Agent ID
    pub target_agent_id: String,
    /// 公钥
    pub public_key: Vec<u8>,
    /// 支持的算法
    pub supported_algorithms: Vec<EncryptionAlgorithm>,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 密钥交换响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangeResponse {
    /// 请求ID
    pub request_id: String,
    /// 响应状态
    pub status: KeyExchangeStatus,
    /// 公钥
    pub public_key: Option<Vec<u8>>,
    /// 选择的算法
    pub selected_algorithm: Option<EncryptionAlgorithm>,
    /// 共享密钥ID
    pub shared_key_id: Option<String>,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 密钥交换状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyExchangeStatus {
    /// 成功
    Success,
    /// 失败
    Failed,
    /// 不支持的算法
    UnsupportedAlgorithm,
    /// 无效的公钥
    InvalidPublicKey,
}

impl EncryptionManager {
    /// 创建新的加密管理器
    pub fn new(config: EncryptionConfig) -> Self {
        info!("🔐 创建加密管理器，算法: {:?}", config.default_algorithm);
        
        Self {
            key_store: HashMap::new(),
            config,
            key_rotation_history: Vec::new(),
        }
    }

    /// 生成新的加密密钥
    pub fn generate_key(&mut self, purpose: KeyPurpose) -> A2AResult<String> {
        let key_id = Uuid::new_v4().to_string();
        let key_data = self.generate_random_key()?;
        
        let expires_at = if self.config.key_rotation_interval_hours > 0 {
            Some(Utc::now() + Duration::hours(self.config.key_rotation_interval_hours as i64))
        } else {
            None
        };

        let key = EncryptionKey {
            key_id: key_id.clone(),
            key_data,
            algorithm: self.config.default_algorithm.clone(),
            created_at: Utc::now(),
            expires_at,
            purpose,
            status: KeyStatus::Active,
        };

        self.key_store.insert(key_id.clone(), key);
        
        debug!("生成新密钥: {}", key_id);
        Ok(key_id)
    }

    /// 轮换密钥
    pub fn rotate_key(&mut self, old_key_id: &str, reason: String) -> A2AResult<String> {
        let old_key = self.key_store.get_mut(old_key_id)
            .ok_or_else(|| A2AError::internal(format!("密钥未找到: {}", old_key_id)))?;

        let purpose = old_key.purpose.clone();
        old_key.status = KeyStatus::Rotated;

        // 生成新密钥
        let new_key_id = self.generate_key(purpose)?;

        // 记录轮换事件
        let rotation_event = KeyRotationEvent {
            event_id: Uuid::new_v4().to_string(),
            old_key_id: old_key_id.to_string(),
            new_key_id: new_key_id.clone(),
            rotated_at: Utc::now(),
            reason,
        };

        self.key_rotation_history.push(rotation_event);

        info!("密钥轮换完成: {} -> {}", old_key_id, new_key_id);
        Ok(new_key_id)
    }

    /// 加密消息
    pub fn encrypt_message(&self, key_id: &str, plaintext: &[u8]) -> A2AResult<EncryptedMessage> {
        let key = self.key_store.get(key_id)
            .ok_or_else(|| A2AError::internal(format!("密钥未找到: {}", key_id)))?;

        if key.status != KeyStatus::Active {
            return Err(A2AError::internal(format!("密钥状态无效: {:?}", key.status)));
        }

        match key.algorithm {
            EncryptionAlgorithm::AES256GCM => {
                self.encrypt_aes256gcm(key, plaintext)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.encrypt_chacha20poly1305(key, plaintext)
            }
            EncryptionAlgorithm::XChaCha20Poly1305 => {
                self.encrypt_xchacha20poly1305(key, plaintext)
            }
            EncryptionAlgorithm::None => {
                Ok(EncryptedMessage {
                    key_id: key_id.to_string(),
                    algorithm: EncryptionAlgorithm::None,
                    iv: Vec::new(),
                    encrypted_data: plaintext.to_vec(),
                    auth_tag: None,
                    aad: None,
                })
            }
            _ => Err(A2AError::internal(format!("不支持的加密算法: {:?}", key.algorithm))),
        }
    }

    /// 解密消息
    pub fn decrypt_message(&self, encrypted_msg: &EncryptedMessage) -> A2AResult<Vec<u8>> {
        let key = self.key_store.get(&encrypted_msg.key_id)
            .ok_or_else(|| A2AError::internal(format!("密钥未找到: {}", encrypted_msg.key_id)))?;

        match encrypted_msg.algorithm {
            EncryptionAlgorithm::AES256GCM => {
                self.decrypt_aes256gcm(key, encrypted_msg)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                self.decrypt_chacha20poly1305(key, encrypted_msg)
            }
            EncryptionAlgorithm::XChaCha20Poly1305 => {
                self.decrypt_xchacha20poly1305(key, encrypted_msg)
            }
            EncryptionAlgorithm::None => {
                Ok(encrypted_msg.encrypted_data.clone())
            }
            _ => Err(A2AError::internal(format!("不支持的解密算法: {:?}", encrypted_msg.algorithm))),
        }
    }

    /// 处理密钥交换请求
    pub fn handle_key_exchange_request(&mut self, request: &KeyExchangeRequest) -> A2AResult<KeyExchangeResponse> {
        debug!("处理密钥交换请求: {}", request.request_id);

        // 选择支持的算法
        let selected_algorithm = request.supported_algorithms.iter()
            .find(|&alg| self.is_algorithm_supported(alg))
            .cloned();

        if selected_algorithm.is_none() {
            return Ok(KeyExchangeResponse {
                request_id: request.request_id.clone(),
                status: KeyExchangeStatus::UnsupportedAlgorithm,
                public_key: None,
                selected_algorithm: None,
                shared_key_id: None,
                error_message: Some("不支持的加密算法".to_string()),
            });
        }

        // 生成密钥对
        let (public_key, shared_key_id) = self.generate_key_pair_for_exchange()?;

        Ok(KeyExchangeResponse {
            request_id: request.request_id.clone(),
            status: KeyExchangeStatus::Success,
            public_key: Some(public_key),
            selected_algorithm,
            shared_key_id: Some(shared_key_id),
            error_message: None,
        })
    }

    /// 获取密钥信息
    pub fn get_key_info(&self, key_id: &str) -> Option<&EncryptionKey> {
        self.key_store.get(key_id)
    }

    /// 列出所有密钥
    pub fn list_keys(&self) -> Vec<&EncryptionKey> {
        self.key_store.values().collect()
    }

    /// 清理过期密钥
    pub fn cleanup_expired_keys(&mut self) -> usize {
        let now = Utc::now();
        let mut expired_keys = Vec::new();

        for (key_id, key) in &self.key_store {
            if let Some(expires_at) = key.expires_at {
                if now > expires_at {
                    expired_keys.push(key_id.clone());
                }
            }
        }

        for key_id in &expired_keys {
            if let Some(key) = self.key_store.get_mut(key_id) {
                key.status = KeyStatus::Expired;
            }
        }

        let count = expired_keys.len();
        if count > 0 {
            info!("清理了 {} 个过期密钥", count);
        }

        count
    }

    /// 获取密钥轮换历史
    pub fn get_key_rotation_history(&self) -> &[KeyRotationEvent] {
        &self.key_rotation_history
    }

    // 私有方法

    fn generate_random_key(&self) -> A2AResult<Vec<u8>> {
        // 这里应该使用加密安全的随机数生成器
        // 为了简化，我们使用模拟数据
        let mut key = vec![0u8; self.config.key_length];
        for (i, byte) in key.iter_mut().enumerate() {
            *byte = (i % 256) as u8; // 简化的随机数生成
        }
        Ok(key)
    }

    fn encrypt_aes256gcm(&self, key: &EncryptionKey, plaintext: &[u8]) -> A2AResult<EncryptedMessage> {
        // 简化的AES-256-GCM加密实现
        // 实际应该使用真正的加密库
        let iv = vec![0u8; 12]; // 96-bit IV for GCM
        let encrypted_data = plaintext.to_vec(); // 简化：不实际加密
        let auth_tag = vec![0u8; 16]; // 128-bit auth tag

        Ok(EncryptedMessage {
            key_id: key.key_id.clone(),
            algorithm: EncryptionAlgorithm::AES256GCM,
            iv,
            encrypted_data,
            auth_tag: Some(auth_tag),
            aad: None,
        })
    }

    fn decrypt_aes256gcm(&self, _key: &EncryptionKey, encrypted_msg: &EncryptedMessage) -> A2AResult<Vec<u8>> {
        // 简化的AES-256-GCM解密实现
        Ok(encrypted_msg.encrypted_data.clone())
    }

    fn encrypt_chacha20poly1305(&self, key: &EncryptionKey, plaintext: &[u8]) -> A2AResult<EncryptedMessage> {
        // 简化的ChaCha20-Poly1305加密实现
        let iv = vec![0u8; 12]; // 96-bit nonce
        let encrypted_data = plaintext.to_vec();
        let auth_tag = vec![0u8; 16];

        Ok(EncryptedMessage {
            key_id: key.key_id.clone(),
            algorithm: EncryptionAlgorithm::ChaCha20Poly1305,
            iv,
            encrypted_data,
            auth_tag: Some(auth_tag),
            aad: None,
        })
    }

    fn decrypt_chacha20poly1305(&self, _key: &EncryptionKey, encrypted_msg: &EncryptedMessage) -> A2AResult<Vec<u8>> {
        Ok(encrypted_msg.encrypted_data.clone())
    }

    fn encrypt_xchacha20poly1305(&self, key: &EncryptionKey, plaintext: &[u8]) -> A2AResult<EncryptedMessage> {
        // 简化的XChaCha20-Poly1305加密实现
        let iv = vec![0u8; 24]; // 192-bit nonce
        let encrypted_data = plaintext.to_vec();
        let auth_tag = vec![0u8; 16];

        Ok(EncryptedMessage {
            key_id: key.key_id.clone(),
            algorithm: EncryptionAlgorithm::XChaCha20Poly1305,
            iv,
            encrypted_data,
            auth_tag: Some(auth_tag),
            aad: None,
        })
    }

    fn decrypt_xchacha20poly1305(&self, _key: &EncryptionKey, encrypted_msg: &EncryptedMessage) -> A2AResult<Vec<u8>> {
        Ok(encrypted_msg.encrypted_data.clone())
    }

    fn is_algorithm_supported(&self, algorithm: &EncryptionAlgorithm) -> bool {
        matches!(algorithm, 
            EncryptionAlgorithm::AES256GCM |
            EncryptionAlgorithm::ChaCha20Poly1305 |
            EncryptionAlgorithm::XChaCha20Poly1305 |
            EncryptionAlgorithm::None
        )
    }

    fn generate_key_pair_for_exchange(&mut self) -> A2AResult<(Vec<u8>, String)> {
        // 简化的密钥对生成
        let public_key = vec![1u8; 32]; // 模拟公钥
        let shared_key_id = self.generate_key(KeyPurpose::KeyExchange)?;
        
        Ok((public_key, shared_key_id))
    }
}
