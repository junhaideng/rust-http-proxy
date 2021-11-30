## HTTP 代理 

[![Rust](https://github.com/junhaideng/rust-http-proxy/actions/workflows/rust.yml/badge.svg)](https://github.com/junhaideng/rust-http-proxy/actions/workflows/rust.yml)

- [x] `HTTP` 协议的解析(包括请求和响应)
- [x] 请求过滤以及响应过滤 (防火墙作用)
- [x] 可配置的代理鉴权 
- [x] 整合 `iptables` 实现透明代理 (参数开关)
- [x] 日志记录
- [x] 测试 (且集成到 `GitHub Actions` 中)
- [x] `HTTPS Tunnel`


### 运行
项目根目录运行即可, 注意 `config.yml` 亦在根目录下
```bash
# 运行
cargo run 

# 构建
cargo build  # debug 版本
cargo build --release 
```

### 配置文件介绍
```yaml
server: 
    # 有关于代理鉴权
    auth: 
        enable: false  # 支持鉴权，否则下面的字段不会使用
        username: rust 
        password: proxy

# 需要被过滤的内容
deny:
    request:  # 针对请求过滤，可以有多个规则，每个规则都匹配才会过滤掉
      - 
        name: request_deny_1
        rule:  
          line: # 请求行中的数据
            methods: [POST, PUT]  # 请求方法，满足其一即可
            path:  # 请求路径，满足其一即可，支持正则匹配
              - /login
          headers:  # 请求头部，全部满足才会
            - 
              key: "Content-Type"
              value: "application/json"
      
        
    response: # 响应中的数据过滤，暂只对请求头部信息进行过滤，全匹配才能被过滤掉
        -
          name: response_deny_1
          rule: 
            headers:
              - 
                key: "Content-Type"
                value: "application/json"
              -
                key: "Access-Control-Allow-Credentials"
                value: "true"

```

转换成 `json` 格式如下
```json
{
  "server": {
    "auth": {
      "enable": false,
      "username": "rust",
      "password": "proxy"
    }
  },
  "deny": {
    "request": [
      {
        "name": "deny_1",
        "rule": {
          "line": {
            "methods": ["POST", "PUT"],
            "path": ["/login"]
          },
          "headers": [
            {
              "key": "Content-Type",
              "value": "application/json"
            }
          ]
        }
      }
    ],
    "response": [
      {
        "name": "deny",
        "rule": {
          "headers": [
            {
              "key": "Content-Type",
              "value": "application/json"
            },
            {
              "key": "Access-Control-Allow-Credentials",
              "value": "true"
            }
          ]
        }
      }
    ]
  }
}

```