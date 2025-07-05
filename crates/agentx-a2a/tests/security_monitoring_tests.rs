//! 安全和监控系统集成测试
//! 
//! 测试加密通信、认证授权、监控面板等功能

use agentx_a2a::*;
use std::collections::HashMap;
use chrono::Utc;

#[tokio::test]
async fn test_encryption_manager() {
    println!("🚀 测试加密管理器");

    let config = EncryptionConfig {
        default_algorithm: EncryptionAlgorithm::AES256GCM,
        key_length: 32,
        key_rotation_interval_hours: 24,
        enable_e2e_encryption: true,
        enable_transport_encryption: true,
        key_derivation_iterations: 100_000,
    };

    let mut encryption_manager = EncryptionManager::new(config);

    // 测试密钥生成
    let key_id = encryption_manager.generate_key(KeyPurpose::MessageEncryption).unwrap();
    assert!(!key_id.is_empty());

    let key_info = encryption_manager.get_key_info(&key_id).unwrap();
    assert_eq!(key_info.algorithm, EncryptionAlgorithm::AES256GCM);
    assert_eq!(key_info.purpose, KeyPurpose::MessageEncryption);
    assert_eq!(key_info.status, KeyStatus::Active);

    // 测试消息加密和解密
    let plaintext = b"Hello, encrypted world!";
    let encrypted_msg = encryption_manager.encrypt_message(&key_id, plaintext).unwrap();
    
    assert_eq!(encrypted_msg.algorithm, EncryptionAlgorithm::AES256GCM);
    assert_eq!(encrypted_msg.key_id, key_id);
    assert!(encrypted_msg.auth_tag.is_some());

    let decrypted = encryption_manager.decrypt_message(&encrypted_msg).unwrap();
    assert_eq!(decrypted, plaintext);

    // 测试密钥轮换
    let new_key_id = encryption_manager.rotate_key(&key_id, "定期轮换".to_string()).unwrap();
    assert_ne!(new_key_id, key_id);

    let old_key = encryption_manager.get_key_info(&key_id).unwrap();
    assert_eq!(old_key.status, KeyStatus::Rotated);

    let new_key = encryption_manager.get_key_info(&new_key_id).unwrap();
    assert_eq!(new_key.status, KeyStatus::Active);

    // 测试密钥轮换历史
    let history = encryption_manager.get_key_rotation_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].old_key_id, key_id);
    assert_eq!(history[0].new_key_id, new_key_id);

    println!("✅ 加密管理器测试通过");
}

#[tokio::test]
async fn test_key_exchange() {
    println!("🚀 测试密钥交换");

    let mut encryption_manager = EncryptionManager::new(EncryptionConfig::default());

    // 创建密钥交换请求
    let request = KeyExchangeRequest {
        request_id: "test_exchange_1".to_string(),
        initiator_agent_id: "agent_1".to_string(),
        target_agent_id: "agent_2".to_string(),
        public_key: vec![1, 2, 3, 4],
        supported_algorithms: vec![
            EncryptionAlgorithm::AES256GCM,
            EncryptionAlgorithm::ChaCha20Poly1305,
        ],
        timestamp: Utc::now(),
    };

    // 处理密钥交换请求
    let response = encryption_manager.handle_key_exchange_request(&request).unwrap();

    assert_eq!(response.request_id, request.request_id);
    assert_eq!(response.status, KeyExchangeStatus::Success);
    assert!(response.public_key.is_some());
    assert!(response.selected_algorithm.is_some());
    assert!(response.shared_key_id.is_some());

    // 验证选择的算法是支持的
    let selected_alg = response.selected_algorithm.unwrap();
    assert!(request.supported_algorithms.contains(&selected_alg));

    println!("✅ 密钥交换测试通过");
}

#[tokio::test]
async fn test_multiple_encryption_algorithms() {
    println!("🚀 测试多种加密算法");

    let algorithms = vec![
        EncryptionAlgorithm::AES256GCM,
        EncryptionAlgorithm::ChaCha20Poly1305,
        EncryptionAlgorithm::XChaCha20Poly1305,
        EncryptionAlgorithm::None,
    ];

    for algorithm in algorithms {
        let config = EncryptionConfig {
            default_algorithm: algorithm.clone(),
            ..Default::default()
        };

        let mut encryption_manager = EncryptionManager::new(config);
        let key_id = encryption_manager.generate_key(KeyPurpose::MessageEncryption).unwrap();

        let plaintext = b"Test message for different algorithms";
        let encrypted_msg = encryption_manager.encrypt_message(&key_id, plaintext).unwrap();
        
        assert_eq!(encrypted_msg.algorithm, algorithm);
        
        let decrypted = encryption_manager.decrypt_message(&encrypted_msg).unwrap();
        assert_eq!(decrypted, plaintext);

        println!("✅ 算法 {:?} 测试通过", algorithm);
    }

    println!("✅ 多种加密算法测试通过");
}

#[tokio::test]
async fn test_monitoring_dashboard() {
    println!("🚀 测试监控面板");

    let monitoring_config = MonitoringConfig {
        metric_retention_hours: 24,
        health_check_interval_seconds: 30,
        stats_calculation_interval_seconds: 60,
        enable_detailed_monitoring: true,
    };

    let monitoring_manager = MonitoringManager::new(monitoring_config);

    let dashboard_config = DashboardConfig {
        refresh_interval_seconds: 30,
        data_retention_hours: 24,
        enable_alerts: true,
        alert_check_interval_seconds: 60,
        max_alerts: 100,
    };

    let mut dashboard = MonitoringDashboard::new(monitoring_manager, dashboard_config);

    // 测试添加告警规则
    let alert_rule = AlertRule {
        rule_id: "cpu_high".to_string(),
        name: "CPU使用率过高".to_string(),
        metric_name: "cpu_usage".to_string(),
        condition: AlertCondition::GreaterThan,
        threshold: 80.0,
        duration_seconds: 300,
        severity: AlertSeverity::Warning,
        enabled: true,
        description: "CPU使用率超过80%".to_string(),
    };

    dashboard.add_alert_rule(alert_rule);

    // 测试添加小部件
    let widget = Widget {
        widget_id: "cpu_chart".to_string(),
        widget_type: WidgetType::LineChart,
        title: "CPU使用率".to_string(),
        data_source: DataSource {
            metric_name: "cpu_usage".to_string(),
            time_range_minutes: 60,
            aggregation: AggregationType::Average,
            filters: HashMap::new(),
        },
        config: WidgetConfig {
            color: Some("#FF6B6B".to_string()),
            unit: Some("%".to_string()),
            decimal_places: Some(1),
            min_value: Some(0.0),
            max_value: Some(100.0),
            thresholds: vec![
                Threshold {
                    value: 70.0,
                    color: "#FFA500".to_string(),
                    label: "警告".to_string(),
                },
                Threshold {
                    value: 90.0,
                    color: "#FF0000".to_string(),
                    label: "危险".to_string(),
                },
            ],
        },
        position: WidgetPosition {
            x: 0,
            y: 0,
            width: 6,
            height: 4,
        },
    };

    dashboard.add_widget(widget);

    // 测试获取面板数据
    let dashboard_data = dashboard.get_dashboard_data().unwrap();
    
    assert!(dashboard_data.system_overview.registered_agents > 0);
    assert!(dashboard_data.performance_metrics.cpu_usage >= 0.0);
    assert!(dashboard_data.performance_metrics.cpu_usage <= 1.0);
    assert!(!dashboard_data.widget_data.is_empty());

    // 测试告警检查
    let new_alerts = dashboard.check_alerts().unwrap();
    // 由于模拟数据，可能会触发告警
    println!("检测到 {} 个新告警", new_alerts.len());

    // 测试小部件管理
    let widgets = dashboard.get_widgets();
    assert_eq!(widgets.len(), 1);
    assert!(widgets.contains_key("cpu_chart"));

    println!("✅ 监控面板测试通过");
}

#[tokio::test]
async fn test_alert_management() {
    println!("🚀 测试告警管理");

    let monitoring_manager = MonitoringManager::new(MonitoringConfig::default());
    let mut dashboard = MonitoringDashboard::new(monitoring_manager, DashboardConfig::default());

    // 添加多个告警规则
    let rules = vec![
        AlertRule {
            rule_id: "cpu_critical".to_string(),
            name: "CPU使用率严重过高".to_string(),
            metric_name: "cpu_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 95.0,
            duration_seconds: 60,
            severity: AlertSeverity::Critical,
            enabled: true,
            description: "CPU使用率超过95%".to_string(),
        },
        AlertRule {
            rule_id: "memory_warning".to_string(),
            name: "内存使用率警告".to_string(),
            metric_name: "memory_usage".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 85.0,
            duration_seconds: 180,
            severity: AlertSeverity::Warning,
            enabled: true,
            description: "内存使用率超过85%".to_string(),
        },
        AlertRule {
            rule_id: "error_rate_high".to_string(),
            name: "错误率过高".to_string(),
            metric_name: "error_rate".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 0.05,
            duration_seconds: 120,
            severity: AlertSeverity::Error,
            enabled: true,
            description: "错误率超过5%".to_string(),
        },
    ];

    for rule in rules {
        dashboard.add_alert_rule(rule);
    }

    // 检查告警
    let alerts = dashboard.check_alerts().unwrap();
    println!("触发了 {} 个告警", alerts.len());

    // 测试告警状态管理
    if !alerts.is_empty() {
        let alert_id = &alerts[0].alert_id;
        
        // 确认告警
        dashboard.acknowledge_alert(alert_id).unwrap();
        let active_alerts = dashboard.get_active_alerts();
        let acknowledged_alert = active_alerts.iter().find(|a| a.alert_id == *alert_id).unwrap();
        assert_eq!(acknowledged_alert.status, AlertStatus::Acknowledged);

        // 解决告警
        dashboard.resolve_alert(alert_id).unwrap();
        let active_alerts = dashboard.get_active_alerts();
        let resolved_alert = active_alerts.iter().find(|a| a.alert_id == *alert_id).unwrap();
        assert_eq!(resolved_alert.status, AlertStatus::Resolved);
        assert!(resolved_alert.resolved_at.is_some());
    }

    println!("✅ 告警管理测试通过");
}

#[tokio::test]
async fn test_widget_types() {
    println!("🚀 测试小部件类型");

    let monitoring_manager = MonitoringManager::new(MonitoringConfig::default());
    let mut dashboard = MonitoringDashboard::new(monitoring_manager, DashboardConfig::default());

    let widget_types = vec![
        WidgetType::LineChart,
        WidgetType::BarChart,
        WidgetType::PieChart,
        WidgetType::Gauge,
        WidgetType::Number,
        WidgetType::Table,
        WidgetType::StatusIndicator,
    ];

    for (i, widget_type) in widget_types.into_iter().enumerate() {
        let widget = Widget {
            widget_id: format!("widget_{}", i),
            widget_type: widget_type.clone(),
            title: format!("测试小部件 {}", i),
            data_source: DataSource {
                metric_name: "test_metric".to_string(),
                time_range_minutes: 30,
                aggregation: AggregationType::Average,
                filters: HashMap::new(),
            },
            config: WidgetConfig {
                color: Some("#4ECDC4".to_string()),
                unit: Some("units".to_string()),
                decimal_places: Some(2),
                min_value: Some(0.0),
                max_value: Some(100.0),
                thresholds: Vec::new(),
            },
            position: WidgetPosition {
                x: (i % 3) as u32 * 4,
                y: (i / 3) as u32 * 3,
                width: 4,
                height: 3,
            },
        };

        dashboard.add_widget(widget);
        println!("✅ 添加了 {:?} 类型的小部件", widget_type);
    }

    // 验证所有小部件都已添加
    let widgets = dashboard.get_widgets();
    assert_eq!(widgets.len(), 7);

    // 获取面板数据，验证小部件数据收集
    let dashboard_data = dashboard.get_dashboard_data().unwrap();
    assert_eq!(dashboard_data.widget_data.len(), 7);

    for (widget_id, widget_data) in &dashboard_data.widget_data {
        assert!(!widget_data.data_points.is_empty());
        assert!(widget_data.current_value.is_some());
        assert!(widget_data.status.is_some());
        println!("✅ 小部件 {} 数据收集正常", widget_id);
    }

    println!("✅ 小部件类型测试通过");
}
