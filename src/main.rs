//#![windows_subsystem = "windows"]
extern crate ssh2;
extern crate web_view;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    mem,
    io::prelude::*,
    net::{TcpStream},
};
use web_view::*;
use ssh2::Session;

fn main() {
    let tcp = TcpStream::connect("192.168.1.251:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "password").unwrap();

    let counter_inner = Arc::new(Mutex::new(String::from("")));
    let service2_inner = Arc::new(Mutex::new(String::from("")));

    let webview = web_view::builder()
        .title("Linux service manager")
        .content(Content::Html(HTML))
        .size(800, 600)
        .resizable(true)
        .debug(true)
        .user_data(0)
        .invoke_handler(|webview, arg| {
            match arg {
                "stop" => {  
                    systemd_command_arg("systemctl stop firewalld")
                }
                "start" => {
                    systemd_command_arg("systemctl start firewalld")
                }
                "stop_tuned" => {  
                    systemd_command_arg("systemctl stop tuned")
                }
                "start_tuned" => {
                    systemd_command_arg("systemctl start tuned")
                }
                _ => unimplemented!(),
            };
            Ok(())
        })
        .build()
        .unwrap();

    let handle = webview.handle();
    thread::spawn(move || loop {
        {
            let mut counter = counter_inner.lock().unwrap();
            let mut service2_counter = service2_inner.lock().unwrap();
            let count;
            let service2_count: String;

            let mut channel = sess.channel_session().unwrap();
            channel.exec("systemctl is-active firewalld").unwrap();
            channel.read_to_string(&mut counter).unwrap();
            if counter.contains("unknown") {
                mem::replace( &mut *counter, String::from("stopped"));
                count = counter.to_string();   
            } else {
                mem::replace( &mut *counter, String::from("started"));
                count = counter.to_string();   
            }

            let mut channel = sess.channel_session().unwrap();
            channel.exec("systemctl is-active tuned").unwrap();
            channel.read_to_string(&mut service2_counter).unwrap();
            if service2_counter.contains("inactive") {
                mem::replace( &mut *service2_counter, String::from("stopped"));
                service2_count = service2_counter.to_string();   
            } else {
                mem::replace( &mut *service2_counter, String::from("started"));
                service2_count = service2_counter.to_string();   
            }

            handle
                .dispatch(move |webview| {
                    render(webview, &count, &service2_count)
                })
                .unwrap();           
        }
        thread::sleep(Duration::from_secs(1));
    });

    webview.run().unwrap();
}

fn render(webview: &mut WebView<i32>, counter: &String, service2_counter: &String) -> WVResult {
    println!("counter: {}, service2_counter: {},", counter, service2_counter);
    webview.eval(&format!("updateTicks({:#?}, {:#?})", counter, service2_counter))
}

fn systemd_command_arg(command_arg: &str){
    let tcp = TcpStream::connect("192.168.1.251:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();
    sess.userauth_password("root", "password").unwrap();                 
    let mut channel = sess.channel_session().unwrap();
    channel.exec(command_arg).unwrap();
    channel.wait_close();
}

const HTML: &str = r#"
<!doctype html>
<html>
    <body>   
        <script type="text/javascript">
			function updateTicks(s1, s2) {
                if (s1 == "stopped") {
                    document.getElementById('ticks').innerHTML = '<br>' + '<br>' + s1.fontcolor("red");
                } else {
                    document.getElementById('ticks').innerHTML = '<br>' + '<br>' + s1.fontcolor("green");
                }
                if (s2 == "stopped") {
                    document.getElementById('service2stat').innerHTML = '<br>' + '<br>' + s2.fontcolor("red");
                } else {
                    document.getElementById('service2stat').innerHTML = '<br>' + '<br>' + s2.fontcolor("green");
                }
			}
        </script>
        <table valign="top">
            <tr>
                <td valign="bottom"><button onclick="external.invoke('start')">start firewall</button></td>
                <td valign="baseline"><div id="ticks"></div></td>
                <td valign="bottom"><button onclick="external.invoke('stop')">stop firewall</button></td>
            </tr>
            <tr>
                <td valign="bottom"><button onclick="external.invoke('start_tuned')">start tuned</button></td>
                <td valign="baseline"><div id="service2stat"></div></td>
                <td valign="bottom""><button onclick="external.invoke('stop_tuned')">stop tuned</button></td>
            </tr>
        </table>
	</body>
</html>
"#;