from pydantic import BaseModel

from flask_openapi3 import Info
from flask_openapi3 import OpenAPI
import subprocess
from enum import Enum

info = Info(title="Arduino Interface API", version="1.0.0")
app = OpenAPI(__name__, info=info)


class Direction(str, Enum):
    forward = "forward"
    backward = "backward"


class Motor(str, Enum):
    a = "a"
    b = "b"


class Device(str, Enum):
    devnull = "/dev/null"


class Buffer(str, Enum):
    period = "period"
    hightime = "hightime"


class ArduinoCommand(BaseModel):
    device: Device | None = None
    motor: Motor
    enabled: bool
    direction: Direction
    buffer: Buffer
    value: int


class ArduinoStop(BaseModel):
    device: Device | None = None
    motor: Motor


@app.post("/command", summary="send a motor command")
def post_command(query: ArduinoCommand):
    """
    post a command to the arduino over serial
    """
    command = ["mount_clt"]
    if query.device is not None:
        command += [f"--device={query.device.value}"]
    command += [
        "full",
        query.motor.value,
        "enabled" if query.enabled else "disabled",
        query.direction.value,
        query.buffer.value,
        f"{query.value}",
    ]
    print(command)
    result = subprocess.run(command, capture_output=True)
    print(result)
    if result.returncode == 0:
        # success
        return {
            "message": "ok",
            "data": [
                "message sent successfully"
            ]
        }
    else:
        # something went wrong
        return {
            "message": result.stderr.decode()
        }, 500


@app.post("/stop", summary="stop a motor")
def post_stop(query: ArduinoStop):
    """
    post a stop-command to the arduino over serial
    """
    command = ["mount_clt"]
    if query.device is not None:
        command += [f"--device={query.device.value}"]
    command += [
        "stop",
        query.motor.value,
    ]
    result = subprocess.run(command, capture_output=True)
    if result.returncode == 0:
        # success
        return {
            "message": "ok",
            "data": [
                "message sent successfully"
            ]
        }
    else:
        # something went wrong
        return {
            "message": result.stderr.decode()
        }, 500


if __name__ == "__main__":
    app.run(debug=True)
