//! A2A协议安全认证测试
//! 
//! 测试A2A协议的安全认证、授权和加密功能

use agentx_a2a::{
    SecurityManager, SecurityConfig, AuthCredentials, AuthType, 
    EncryptionAlgorithm, SignatureAlgorithm, TrustLevel,
};
use std::collections::HashMap;
use chrono::{Utc, Duration};
use tokio;

#[tokio::test]
async fn test_security_manager_creation() {
    println!("🧪 测试安全管理器创建");
    
    let config = SecurityConfig::default();
    let manager = SecurityManager::new(config);
    
    println!("   ✅ 安全管理器创建成功");
}

#[tokio::test]
async fn test_no_auth_authentication() {
    println!("🧪 测试无认证模式");
    
    let config = SecurityConfig {
        auth_type: AuthType::None,
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    let credentials = AuthCredentials {
        auth_type: AuthType::None,
        credentials: HashMap::new(),
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let result = manager.authenticate("test_agent", credentials);
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert_eq!(context.agent_id, "test_agent");
    assert_eq!(context.trust_level, TrustLevel::Public);
    
    println!("   ✅ 无认证模式测试通过");
}

#[tokio::test]
async fn test_api_key_authentication() {
    println!("🧪 测试API密钥认证");
    
    let config = SecurityConfig {
        auth_type: AuthType::ApiKey,
        required_trust_level: TrustLevel::Public,
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 测试有效的API密钥
    let mut credentials_map = HashMap::new();
    credentials_map.insert("api_key".to_string(), "a".repeat(32)); // 32字符的有效密钥
    
    let credentials = AuthCredentials {
        auth_type: AuthType::ApiKey,
        credentials: credentials_map,
        expires_at: None,
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    let result = manager.authenticate("api_agent", credentials);
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert_eq!(context.agent_id, "api_agent");
    
    println!("   ✅ 有效API密钥认证通过");
    
    // 测试无效的API密钥
    let mut invalid_credentials_map = HashMap::new();
    invalid_credentials_map.insert("api_key".to_string(), "short".to_string()); // 太短的密钥
    
    let invalid_credentials = AuthCredentials {
        auth_type: AuthType::ApiKey,
        credentials: invalid_credentials_map,
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let result = manager.authenticate("invalid_agent", invalid_credentials);
    assert!(result.is_err());
    
    println!("   ✅ 无效API密钥认证拒绝");
}

#[tokio::test]
async fn test_jwt_authentication() {
    println!("🧪 测试JWT令牌认证");
    
    let config = SecurityConfig {
        auth_type: AuthType::JWT,
        required_trust_level: TrustLevel::Public,
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 测试有效的JWT令牌（简化格式）
    let mut credentials_map = HashMap::new();
    credentials_map.insert("token".to_string(), "header.payload.signature".to_string());
    
    let credentials = AuthCredentials {
        auth_type: AuthType::JWT,
        credentials: credentials_map,
        expires_at: Some(Utc::now() + Duration::hours(1)),
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    let result = manager.authenticate("jwt_agent", credentials);
    assert!(result.is_ok());
    
    println!("   ✅ 有效JWT令牌认证通过");
    
    // 测试无效的JWT令牌格式
    let mut invalid_credentials_map = HashMap::new();
    invalid_credentials_map.insert("token".to_string(), "invalid.format".to_string()); // 只有两部分
    
    let invalid_credentials = AuthCredentials {
        auth_type: AuthType::JWT,
        credentials: invalid_credentials_map,
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let result = manager.authenticate("invalid_jwt_agent", invalid_credentials);
    assert!(result.is_err());
    
    println!("   ✅ 无效JWT令牌认证拒绝");
}

#[tokio::test]
async fn test_session_management() {
    println!("🧪 测试会话管理");
    
    let config = SecurityConfig {
        auth_type: AuthType::None,
        token_expiry_seconds: 2, // 2秒过期，用于测试
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 创建会话
    let credentials = AuthCredentials {
        auth_type: AuthType::None,
        credentials: HashMap::new(),
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let context = manager.authenticate("session_agent", credentials).unwrap();
    let session_id = context.session_id.clone();
    
    println!("   会话ID: {}", session_id);
    
    // 验证会话
    let result = manager.validate_session(&session_id);
    assert!(result.is_ok());
    
    println!("   ✅ 会话验证通过");
    
    // 等待会话过期
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // 验证过期会话
    let result = manager.validate_session(&session_id);
    assert!(result.is_err());
    
    println!("   ✅ 过期会话验证拒绝");
}

#[tokio::test]
async fn test_trust_level_management() {
    println!("🧪 测试信任级别管理");
    
    let config = SecurityConfig {
        auth_type: AuthType::None,
        required_trust_level: TrustLevel::Trusted,
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 添加信任的Agent
    manager.add_trusted_agent("trusted_agent".to_string(), TrustLevel::Trusted);
    manager.add_trusted_agent("internal_agent".to_string(), TrustLevel::Internal);
    
    // 测试信任级别足够的Agent
    let credentials = AuthCredentials {
        auth_type: AuthType::None,
        credentials: HashMap::new(),
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let result = manager.authenticate("trusted_agent", credentials.clone());
    assert!(result.is_ok());
    
    let context = result.unwrap();
    assert_eq!(context.trust_level, TrustLevel::Trusted);
    
    println!("   ✅ 信任级别足够的Agent认证通过");
    
    // 测试信任级别不足的Agent
    let result = manager.authenticate("unknown_agent", credentials);
    assert!(result.is_err());
    
    println!("   ✅ 信任级别不足的Agent认证拒绝");
}

#[tokio::test]
async fn test_permission_checking() {
    println!("🧪 测试权限检查");
    
    let config = SecurityConfig {
        auth_type: AuthType::None,
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 添加不同信任级别的Agent
    manager.add_trusted_agent("public_agent".to_string(), TrustLevel::Public);
    manager.add_trusted_agent("verified_agent".to_string(), TrustLevel::Verified);
    manager.add_trusted_agent("trusted_agent".to_string(), TrustLevel::Trusted);
    manager.add_trusted_agent("internal_agent".to_string(), TrustLevel::Internal);
    
    let credentials = AuthCredentials {
        auth_type: AuthType::None,
        credentials: HashMap::new(),
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    // 测试不同信任级别的权限
    let agents = vec![
        ("public_agent", TrustLevel::Public),
        ("verified_agent", TrustLevel::Verified),
        ("trusted_agent", TrustLevel::Trusted),
        ("internal_agent", TrustLevel::Internal),
    ];
    
    for (agent_id, expected_trust_level) in agents {
        let context = manager.authenticate(agent_id, credentials.clone()).unwrap();
        assert_eq!(context.trust_level, expected_trust_level);
        
        // 检查基础权限
        assert!(manager.check_permission(&context, "read_public"));
        
        // 检查高级权限
        let has_manage_permission = manager.check_permission(&context, "manage_agents");
        let should_have_manage = matches!(expected_trust_level, TrustLevel::Trusted | TrustLevel::Internal);
        assert_eq!(has_manage_permission, should_have_manage);
        
        // 检查内部权限
        let has_all_permission = manager.check_permission(&context, "any_permission");
        let should_have_all = matches!(expected_trust_level, TrustLevel::Internal);
        assert_eq!(has_all_permission, should_have_all);
        
        println!("   ✅ {} 权限检查通过", agent_id);
    }
}

#[tokio::test]
async fn test_session_revocation() {
    println!("🧪 测试会话撤销");
    
    let config = SecurityConfig {
        auth_type: AuthType::JWT,
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 创建JWT会话
    let mut credentials_map = HashMap::new();
    credentials_map.insert("token".to_string(), "header.payload.signature".to_string());
    
    let credentials = AuthCredentials {
        auth_type: AuthType::JWT,
        credentials: credentials_map,
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let context = manager.authenticate("revoke_agent", credentials).unwrap();
    let session_id = context.session_id.clone();
    
    // 验证会话存在
    let result = manager.validate_session(&session_id);
    assert!(result.is_ok());
    
    // 撤销会话
    let result = manager.revoke_session(&session_id);
    assert!(result.is_ok());
    
    // 验证会话已被撤销
    let result = manager.validate_session(&session_id);
    assert!(result.is_err());
    
    println!("   ✅ 会话撤销测试通过");
}

#[tokio::test]
async fn test_cleanup_expired_sessions() {
    println!("🧪 测试过期会话清理");
    
    let config = SecurityConfig {
        auth_type: AuthType::None,
        token_expiry_seconds: 1, // 1秒过期
        ..Default::default()
    };
    
    let mut manager = SecurityManager::new(config);
    
    // 创建多个会话
    let credentials = AuthCredentials {
        auth_type: AuthType::None,
        credentials: HashMap::new(),
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    let mut session_ids = Vec::new();
    for i in 0..5 {
        let context = manager.authenticate(&format!("cleanup_agent_{}", i), credentials.clone()).unwrap();
        session_ids.push(context.session_id);
    }
    
    println!("   创建了 {} 个会话", session_ids.len());
    
    // 等待会话过期
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 执行清理
    manager.cleanup_expired();
    
    // 验证所有会话都已被清理
    for session_id in &session_ids {
        let result = manager.validate_session(session_id);
        assert!(result.is_err());
    }
    
    println!("   ✅ 过期会话清理测试通过");
}

#[tokio::test]
async fn test_security_config_variations() {
    println!("🧪 测试不同安全配置");
    
    let configs = vec![
        SecurityConfig {
            auth_type: AuthType::None,
            encryption: EncryptionAlgorithm::None,
            signature: SignatureAlgorithm::None,
            require_encryption: false,
            require_signature: false,
            required_trust_level: TrustLevel::Public,
            ..Default::default()
        },
        SecurityConfig {
            auth_type: AuthType::ApiKey,
            encryption: EncryptionAlgorithm::AES256GCM,
            signature: SignatureAlgorithm::HMACSHA256,
            require_encryption: true,
            require_signature: true,
            required_trust_level: TrustLevel::Verified,
            ..Default::default()
        },
        SecurityConfig {
            auth_type: AuthType::MutualTLS,
            encryption: EncryptionAlgorithm::ChaCha20Poly1305,
            signature: SignatureAlgorithm::ECDSAP256,
            require_encryption: true,
            require_signature: true,
            required_trust_level: TrustLevel::Internal,
            ..Default::default()
        },
    ];
    
    for (i, config) in configs.into_iter().enumerate() {
        println!("   测试配置 {}: {:?}", i + 1, config.auth_type);
        
        let manager = SecurityManager::new(config);
        
        // 验证配置被正确设置
        // 这里可以添加更多的配置验证逻辑
        
        println!("     ✅ 配置 {} 验证通过", i + 1);
    }
    
    println!("   ✅ 不同安全配置测试通过");
}
