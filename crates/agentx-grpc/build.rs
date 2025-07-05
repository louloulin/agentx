//! gRPC代码生成构建脚本
//! 
//! 为A2A协议生成gRPC服务和客户端代码

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");
    
    // 编译A2A协议的protobuf定义
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(
            &[
                "proto/a2a_service.proto",
                "proto/plugin_service.proto",
                "proto/agent_registry.proto",
            ],
            &["proto"],
        )?;
    
    Ok(())
}
