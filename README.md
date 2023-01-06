# clifana
Do prometheus visualization stuff on the cli

## ideas
  - use a toml config file
    - config file will have a place to specify multiple data sources in easy-to-connect formats
    ```
    [[server]]
    name=myserver
    url=http://my-prometheus-server:9090/
    ```
    - config file will have ability to store common queries that you use a lot
    ```
    [[query]]
    name=cpu
    query = """
       sum(cpu_seconds_total{foo="$bar"})
    """
    ```
    - config file will support cli argument to variable substitution (or maybe envvar?)
    ```
    [[server]]
    name=generic
    url="http://${serverip}:9090"
    --
    ./clifana --connect=server --query=query --args=serverip,127.0.0.1
    ```
  - maybe visualize graphs in ascii art similar to the python cli visualization tools that exist already
