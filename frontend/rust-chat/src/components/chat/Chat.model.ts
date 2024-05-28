import { TUser } from "../../App.context";
import { GlobalChatMessage } from "../../socket/websocket.model";
import styles from "./Chat.module.css";

export function getStyle(message: GlobalChatMessage, localUser: TUser) {
  const isLocalSender = localUser.id === message.sender.id;
  return cssClassConstructor([
    styles.message,
    isLocalSender ? styles.localSentMessage : "",
  ]);
}
function cssClassConstructor(classes: string[]): string {
  return classes
    .filter((className) => className !== "")
    .join(" ")
    .trim();
}
