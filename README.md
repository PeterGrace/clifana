# clifana
Do prometheus visualization stuff on the cli

## ideas
- use a toml config file
  - config file will have a place to specify multiple data sources in easy-to-connect formats
  ```
  [[myserver]]
  url=http://my-prometheus-server:9090/
  ```
