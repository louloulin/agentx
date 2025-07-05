//! gRPC代码生成构建脚本
//! 
//! 为A2A协议生成gRPC服务和客户端代码

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/");

    // 编译新的AgentX插件协议
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path("proto/descriptor.bin")
        .compile(
            &[
                "proto/agentx_plugin.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
