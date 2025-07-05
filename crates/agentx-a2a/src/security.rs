//! A2A协议安全认证模块
//! 
//! 实现A2A协议的安全认证、授权和加密功能

use crate::{A2AError, A2AResult, TrustLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    /// 无认证
    None,
    /// API密钥认证
    ApiKey,
    /// JWT令牌认证
    JWT,
    /// OAuth 2.0认证
    OAuth2,
    /// 相互TLS认证
    MutualTLS,
    /// 数字签名认证
    DigitalSignature,
    /// 自定义认证
    Custom(String),
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
}

/// 签名算法
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureAlgorithm {
    /// 无签名
    None,
    /// HMAC-SHA256
    HMACSHA256,
    /// RSA-PSS
    RSAPSS,
    /// ECDSA-P256
    ECDSAP256,
    /// Ed25519
    Ed25519,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 认证类型
    pub auth_type: AuthType,
    /// 加密算法
    pub encryption: EncryptionAlgorithm,
    /// 签名算法
    pub signature: SignatureAlgorithm,
    /// 是否要求加密
    pub require_encryption: bool,
    /// 是否要求签名
    pub require_signature: bool,
    /// 令牌过期时间（秒）
    pub token_expiry_seconds: u64,
    /// 最大时钟偏差（秒）
    pub max_clock_skew_seconds: u64,
    /// 信任级别要求
    pub required_trust_level: TrustLevel,
}

/// 认证凭据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    /// 认证类型
    pub auth_type: AuthType,
    /// 凭据数据
    pub credentials: HashMap<String, String>,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 权限范围
    pub scopes: Vec<String>,
}

/// 安全上下文
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Agent ID
    pub agent_id: String,
    /// 认证凭据
    pub credentials: Option<AuthCredentials>,
    /// 信任级别
    pub trust_level: TrustLevel,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 会话ID
    pub session_id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活动时间
    pub last_activity: DateTime<Utc>,
}

/// 安全管理器
#[derive(Debug)]
pub struct SecurityManager {
    /// 安全配置
    config: SecurityConfig,
    /// 活跃会话
    active_sessions: HashMap<String, SecurityContext>,
    /// 信任的Agent列表
    trusted_agents: HashMap<String, TrustLevel>,
    /// 撤销的令牌列表
    revoked_tokens: HashMap<String, DateTime<Utc>>,
}

impl SecurityManager {
    /// 创建新的安全管理器
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config,
            active_sessions: HashMap::new(),
            trusted_agents: HashMap::new(),
            revoked_tokens: HashMap::new(),
        }
    }
    
    /// 验证认证凭据
    pub fn authenticate(&mut self, agent_id: &str, credentials: AuthCredentials) -> A2AResult<SecurityContext> {
        // 检查认证类型是否匹配
        if credentials.auth_type != self.config.auth_type && self.config.auth_type != AuthType::None {
            return Err(A2AError::authentication(
                format!("不支持的认证类型: {:?}", credentials.auth_type)
            ));
        }
        
        // 检查凭据是否过期
        if let Some(expires_at) = credentials.expires_at {
            if Utc::now() > expires_at {
                return Err(A2AError::authentication("凭据已过期"));
            }
        }
        
        // 验证具体的认证方式
        match credentials.auth_type {
            AuthType::None => {
                // 无认证，直接通过
            },
            AuthType::ApiKey => {
                self.validate_api_key(&credentials)?;
            },
            AuthType::JWT => {
                self.validate_jwt_token(&credentials)?;
            },
            AuthType::OAuth2 => {
                self.validate_oauth2_token(&credentials)?;
            },
            AuthType::MutualTLS => {
                self.validate_mtls_certificate(&credentials)?;
            },
            AuthType::DigitalSignature => {
                self.validate_digital_signature(&credentials)?;
            },
            AuthType::Custom(ref method) => {
                self.validate_custom_auth(method, &credentials)?;
            },
        }
        
        // 获取Agent的信任级别
        let trust_level = self.trusted_agents.get(agent_id)
            .copied()
            .unwrap_or(TrustLevel::Public);
        
        // 检查信任级别是否满足要求
        if trust_level.trust_score() < self.config.required_trust_level.trust_score() {
            return Err(A2AError::authorization(
                format!("信任级别不足: 需要 {:?}，当前 {:?}",
                       self.config.required_trust_level, trust_level)
            ));
        }
        
        // 创建安全上下文
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let context = SecurityContext {
            agent_id: agent_id.to_string(),
            credentials: Some(credentials),
            trust_level,
            permissions: self.get_permissions_for_trust_level(trust_level),
            session_id: session_id.clone(),
            created_at: now,
            last_activity: now,
        };
        
        // 存储会话
        self.active_sessions.insert(session_id, context.clone());
        
        Ok(context)
    }
    
    /// 验证会话
    pub fn validate_session(&mut self, session_id: &str) -> A2AResult<&mut SecurityContext> {
        // 首先检查会话是否存在
        if !self.active_sessions.contains_key(session_id) {
            return Err(A2AError::authentication("无效的会话ID"));
        }

        // 检查会话是否过期
        let now = Utc::now();
        let max_session_duration = Duration::seconds(self.config.token_expiry_seconds as i64);

        let should_remove = {
            let context = self.active_sessions.get(session_id).unwrap();
            let session_duration = now.signed_duration_since(context.created_at);
            session_duration > max_session_duration
        };

        if should_remove {
            self.active_sessions.remove(session_id);
            return Err(A2AError::authentication("会话已过期"));
        }

        // 更新最后活动时间并返回上下文
        let context = self.active_sessions.get_mut(session_id).unwrap();
        context.last_activity = now;

        Ok(context)
    }
    
    /// 检查权限
    pub fn check_permission(&self, context: &SecurityContext, permission: &str) -> bool {
        context.permissions.contains(&permission.to_string()) ||
        context.permissions.contains(&"*".to_string())
    }
    
    /// 撤销会话
    pub fn revoke_session(&mut self, session_id: &str) -> A2AResult<()> {
        if let Some(context) = self.active_sessions.remove(session_id) {
            // 如果有JWT令牌，将其加入撤销列表
            if let Some(credentials) = &context.credentials {
                if credentials.auth_type == AuthType::JWT {
                    if let Some(token) = credentials.credentials.get("token") {
                        self.revoked_tokens.insert(token.clone(), Utc::now());
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 添加信任的Agent
    pub fn add_trusted_agent(&mut self, agent_id: String, trust_level: TrustLevel) {
        self.trusted_agents.insert(agent_id, trust_level);
    }
    
    /// 移除信任的Agent
    pub fn remove_trusted_agent(&mut self, agent_id: &str) {
        self.trusted_agents.remove(agent_id);
    }
    
    /// 清理过期的会话和令牌
    pub fn cleanup_expired(&mut self) {
        let now = Utc::now();
        let max_session_duration = Duration::seconds(self.config.token_expiry_seconds as i64);
        let max_revoked_token_age = Duration::days(7);
        
        // 清理过期会话
        self.active_sessions.retain(|_, context| {
            now.signed_duration_since(context.created_at) <= max_session_duration
        });
        
        // 清理过期的撤销令牌
        self.revoked_tokens.retain(|_, revoked_at| {
            now.signed_duration_since(*revoked_at) <= max_revoked_token_age
        });
    }
    
    // 私有方法 - 验证不同类型的认证
    
    fn validate_api_key(&self, credentials: &AuthCredentials) -> A2AResult<()> {
        let api_key = credentials.credentials.get("api_key")
            .ok_or_else(|| A2AError::authentication("缺少API密钥"))?;

        // 这里应该验证API密钥的有效性
        // 简化实现：检查密钥格式
        if api_key.len() < 32 {
            return Err(A2AError::authentication("API密钥格式无效"));
        }

        Ok(())
    }
    
    fn validate_jwt_token(&self, credentials: &AuthCredentials) -> A2AResult<()> {
        let token = credentials.credentials.get("token")
            .ok_or_else(|| A2AError::authentication("缺少JWT令牌"))?;

        // 检查令牌是否被撤销
        if self.revoked_tokens.contains_key(token) {
            return Err(A2AError::authentication("令牌已被撤销"));
        }

        // 这里应该验证JWT令牌的签名和内容
        // 简化实现：检查令牌格式
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(A2AError::authentication("JWT令牌格式无效"));
        }

        Ok(())
    }
    
    fn validate_oauth2_token(&self, credentials: &AuthCredentials) -> A2AResult<()> {
        let access_token = credentials.credentials.get("access_token")
            .ok_or_else(|| A2AError::authentication("缺少OAuth2访问令牌"))?;

        // 这里应该向OAuth2服务器验证令牌
        // 简化实现：检查令牌格式
        if access_token.is_empty() {
            return Err(A2AError::authentication("OAuth2令牌为空"));
        }

        Ok(())
    }
    
    fn validate_mtls_certificate(&self, credentials: &AuthCredentials) -> A2AResult<()> {
        let cert_fingerprint = credentials.credentials.get("cert_fingerprint")
            .ok_or_else(|| A2AError::authentication("缺少证书指纹"))?;

        // 这里应该验证客户端证书
        // 简化实现：检查指纹格式
        if cert_fingerprint.len() != 64 {
            return Err(A2AError::authentication("证书指纹格式无效"));
        }

        Ok(())
    }
    
    fn validate_digital_signature(&self, credentials: &AuthCredentials) -> A2AResult<()> {
        let signature = credentials.credentials.get("signature")
            .ok_or_else(|| A2AError::authentication("缺少数字签名"))?;

        let public_key = credentials.credentials.get("public_key")
            .ok_or_else(|| A2AError::authentication("缺少公钥"))?;

        // 这里应该验证数字签名
        // 简化实现：检查签名和公钥格式
        if signature.is_empty() || public_key.is_empty() {
            return Err(A2AError::authentication("签名或公钥为空"));
        }

        Ok(())
    }
    
    fn validate_custom_auth(&self, method: &str, credentials: &AuthCredentials) -> A2AResult<()> {
        // 这里应该根据自定义方法验证认证
        // 简化实现：检查是否有必要的凭据
        if credentials.credentials.is_empty() {
            return Err(A2AError::authentication(
                format!("自定义认证方法 {} 缺少凭据", method)
            ));
        }

        Ok(())
    }
    
    fn get_permissions_for_trust_level(&self, trust_level: TrustLevel) -> Vec<String> {
        match trust_level {
            TrustLevel::Public => vec![
                "read_public".to_string(),
                "send_message".to_string(),
            ],
            TrustLevel::Verified => vec![
                "read_public".to_string(),
                "read_verified".to_string(),
                "send_message".to_string(),
                "create_task".to_string(),
            ],
            TrustLevel::Trusted => vec![
                "read_public".to_string(),
                "read_verified".to_string(),
                "read_trusted".to_string(),
                "send_message".to_string(),
                "create_task".to_string(),
                "manage_agents".to_string(),
            ],
            TrustLevel::Internal => vec![
                "*".to_string(), // 所有权限
            ],
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_type: AuthType::None,
            encryption: EncryptionAlgorithm::None,
            signature: SignatureAlgorithm::None,
            require_encryption: false,
            require_signature: false,
            token_expiry_seconds: 3600, // 1小时
            max_clock_skew_seconds: 300, // 5分钟
            required_trust_level: TrustLevel::Public,
        }
    }
}
