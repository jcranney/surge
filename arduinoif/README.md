# Arduino Interface Service `surge-arduinoif`

The purpose of this service is to receive commands from external processes, and then translate/communicate them with the arduino uno over serial port.

## Requirements
On top of the python requirements, there is also the command line tool: `mount_clt`, which is available at [jcranney/mount-clt](https://github.com/jcranney/mount-clt).

## Usage
```bash
flask --app=main.py run --host=0.0.0.0 --port=5000
```
