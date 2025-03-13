use crate::{
    utils::{is_valid_ip, is_valid_port},
    Config,
};
use anyhow::Result;
use clap::Parser;
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'i', long)]
    pub interactive: bool,

    #[arg(long)]
    pub webdav_url: Option<String>,

    #[arg(long)]
    pub webdav_username: Option<String>,

    #[arg(long)]
    pub webdav_password: Option<String>,

    #[arg(long)]
    pub server: bool,

    #[arg(long)]
    pub server_ip: Option<String>,

    #[arg(long)]
    pub server_port: Option<u16>,
}

pub fn parse_args() -> Args {
    Args::parse()
}

#[allow(dead_code)]
pub fn interactive_input(config: &mut Config) -> Result<()> {
    println!("{}", style("欢迎使用配置向导！").cyan().bold());

    let theme = ColorfulTheme::default();

    let sync_options = vec!["WebSocket", "WebDAV"];
    let sync_selection = Select::with_theme(&theme)
        .with_prompt("请选择同步方式")
        .default(0)
        .items(&sync_options)
        .interact()?;

    if sync_selection == 0 {
        // WebSocket 配置
        let server_port = config.webserver_port.unwrap_or(8113);

        config.webserver_port = Some(
            Input::with_theme(&theme)
                .with_prompt("请输入本机服务端口")
                .default(server_port)
                .validate_with(|input: &u16| -> Result<(), String> {
                    if is_valid_port(*input) {
                        Ok(())
                    } else {
                        Err("无效的端口, 请输入 1024 到 65535 之间的数字".to_string())
                    }
                })
                .interact()?,
        );

        // 是否连接到另一台设备
        let is_connect_to_other_device = Confirm::with_theme(&theme)
            .with_prompt("是否连接到另一台设备？")
            .default(config.enable_websocket.unwrap_or(true))
            .interact()?;
        
        if is_connect_to_other_device {
            let peer_device_port = config.peer_device_port.unwrap_or(8113);
            let peer_device_ip = config.peer_device_addr.clone().unwrap_or("".to_string());
            
            // 对等设备
            let peer_device_ip: String = Input::with_theme(&theme)
                .with_prompt("请输入对等设备 IP")
                .default(peer_device_ip)
                .validate_with(|input: &String| -> Result<(), String> {
                    if is_valid_ip(input) {
                        Ok(())
                    } else {
                        Err("无效的 IP 地址".to_string())
                    }
                })
                .interact_text()?;

            let peer_device_port: u16 = Input::with_theme(&theme)
                .with_prompt("请输入对等设备端口")
                .default(peer_device_port)
                .validate_with(|input: &u16| -> Result<(), String> {
                    if is_valid_port(*input) {
                        Ok(())
                    } else {
                        Err("无效的端口, 请查看对等设备的端口号".to_string())
                    }
                })
                .interact()?;

            // 添加对等设备
            config.peer_device_addr = Some(peer_device_ip);
            config.peer_device_port = Some(peer_device_port);
        }
    } else {
        // WebDAV 配置
        config.webdav_url = Some(
            Input::with_theme(&theme)
                .with_prompt("请输入 WebDAV URL")
                .interact_text()?,
        );

        config.username = Some(
            Input::with_theme(&theme)
                .with_prompt("请输入用户名")
                .interact_text()?,
        );

        config.password = Some(
            Password::with_theme(&theme)
                .with_prompt("请输入密码")
                .with_confirmation("请确认密码", "密码不匹配")
                .interact()?,
        );
    }

    // 新的配置完成提示
    println!("\n{}", style("配置完成！").green().bold());
    println!(
        "{}",
        style("===========================================").cyan()
    );
    println!(
        "{}  {}",
        Emoji("🎉", "!"),
        style("恭喜！您的 UniClipboard 已准备就绪！").green().bold()
    );
    println!(
        "{}  {}",
        Emoji("💡", "*"),
        style("提示：随时使用 --help 查看更多选项").italic()
    );
    println!(
        "{}",
        style("===========================================").cyan()
    );

    Ok(())
}
