import { TUser } from "../App.context";
import { MessageType } from "./websocket.model";

export class Socket {
  websocket: WebSocket | undefined;
  isInit = false;

  async init(user: TUser, callback: () => void) {
    this.isInit = true;
    return new Promise((resolve, reject) => {
      this.websocket = new WebSocket(
        `ws://127.0.0.1:8080?username=${user.name}&id=${user.id}`
      );

      this.websocket.onopen = () => {
        console.log("WebSocket connected");
        callback();
        resolve(this.websocket);
      };

      this.websocket.onerror = (error) => {
        console.error("WebSocket connection error:", error);
        reject(error);
      };

      this.websocket.onclose = (event) => {
        if (!event.wasClean) {
          console.warn(
            "WebSocket closed unexpectedly during connection:",
            event
          );
          reject(new Error("WebSocket closed unexpectedly during connection"));
        }
      };
    });
  }

  send(message: MessageType) {
    if (!this.websocket || this.websocket.readyState !== this.websocket.OPEN) {
      throw Error("Tried to send message but not connected to ws server");
    }
    this.websocket.send(JSON.stringify(message));
  }
}
