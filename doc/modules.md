# Modules

Lightkeeper aims to be modular so it's easier to extend and customize.  
The main types of modules are:  
- **Monitoring modules (or monitors):** They gather information and often report status information.
- **Command modules:** They execute commands. Usually through clickable buttons in the UI. Commands often depend on monitors.
E.g. you need a monitor `systemd-service` to be able to use command `systemd-service-start` which allows you to start individual services.
- **Connector modules:** Most monitors and commands use the `ssh`-connector, which sends commands over SSH. Usually you don't need custom connector modules,
but you do need to configure e.g. the `ssh`-connector before it can work. For example, you need to set the username and private key path or password for successful login.


