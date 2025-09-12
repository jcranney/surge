from pydantic import BaseModel

from flask_openapi3 import Info
from flask_openapi3 import OpenAPI
import subprocess

info = Info(title="Arduino Interface API", version="1.0.0")
app = OpenAPI(__name__, info=info)


class ArduinoCommand(BaseModel):
    motor: str
    enabled: bool


@app.post("/command", summary="send a motor command")
def post_command(query: ArduinoCommand):
    """
    post a command to the arduino over serial
    """
    result = subprocess.run(["mount_clt", "--help"])
    print(result)
    return {
        "code": 0,
        "message": "ok",
        "data": [
            {"bid": 1, "age": query.motor, "author": query.enabled},
            {"bid": 2, "age": query.motor, "author": query.enabled}
        ]
    }


if __name__ == "__main__":
    app.run(debug=True)
