proxyIP = 192.168.0.22
proxyPORT = 4000

serverIP = 192.168.0.53
serverPORT = 5000

logHOST = 192.168.0.22
logPORT = 9100

build:
	cargo clean
	cargo build


client:
				cargo build --bin client
				./target/debug/client --target-ip $(serverIP) \
								--target-port $(serverPORT) \
								--timeout 2 \
								--max-retries 5 \
								--log-host $(logHOST) \
								--log-port $(logPORT)


server:
				cargo build --bin server
				./target/debug/server --listen-ip $(serverIP) \
								--listen-port $(serverPORT) \
								--log-host $(logHOST) \
								--log-port $(logPORT)

proxy:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0 \
                --client-delay 0 \
                --server-delay 0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-50-delay-client:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0 \
                --client-delay 0.5 \
                --server-delay 0 \
                --client-delay-time-min 2500 \
                --client-delay-time-max 3500 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-50-delay-server:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0 \
                --client-delay 0 \
                --server-delay 0.5 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 2500 \
                --server-delay-time-max 3500 \
								--log-port $(logPORT)

proxy-100-delay-server:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0 \
                --client-delay 0 \
                --server-delay 1.0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 2500 \
                --server-delay-time-max 3500 \
								--log-port $(logPORT)

proxy-100-delay-client:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0 \
                --client-delay 1.0 \
                --server-delay 0 \
                --client-delay-time-min 2500 \
                --client-delay-time-max 3500 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-50-drop-client:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0.5 \
                --server-drop 0 \
                --client-delay 0 \
                --server-delay 0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-50-drop-server:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0.5 \
                --client-delay 0 \
                --server-delay 0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-100-drop-server:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 1.0 \
                --client-delay 0 \
                --server-delay 0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-100-drop-client:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 1.0 \
                --server-drop 0 \
                --client-delay 0 \
                --server-delay 0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-50-drop-both:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0.5 \
                --server-drop 0.5 \
                --client-delay 0 \
                --server-delay 0 \
                --client-delay-time-min 0 \
                --client-delay-time-max 0 \
                --server-delay-time-min 0 \
                --server-delay-time-max 0 \
								--log-port $(logPORT)

proxy-50-delay-both:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0 \
                --server-drop 0 \
                --client-delay 0.5 \
                --server-delay 0.5 \
                --client-delay-time-min 2500 \
                --client-delay-time-max 3500 \
                --server-delay-time-min 2500 \
                --server-delay-time-max 3500 \
								--log-port $(logPORT)

proxy-50-delay-drop-both:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 0.5 \
                --server-drop 0.5 \
                --client-delay 0.5 \
                --server-delay 0.5 \
                --client-delay-time-min 2500 \
                --client-delay-time-max 3500 \
                --server-delay-time-min 2500 \
                --server-delay-time-max 3500 \
								--log-port $(logPORT)

proxy-100-delay-drop-both:
				cargo build --bin proxy
				./target/debug/proxy --listen-ip $(proxyIP) \
                --listen-port $(proxyPORT) \
                --target-ip $(serverIP) \
                --target-port $(serverPORT) \
                --client-drop 1.0 \
                --server-drop 1.0 \
                --client-delay 1.0 \
                --server-delay 1.0 \
                --client-delay-time-min 2500 \
                --client-delay-time-max 3500 \
                --server-delay-time-min 2500 \
                --server-delay-time-max 3500 \
								--log-port $(logPORT)
















