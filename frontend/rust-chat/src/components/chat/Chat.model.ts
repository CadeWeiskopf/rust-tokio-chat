import { TUser } from "../../App.context";
import styles from "./Chat.module.css";

export enum MessageType {
  DEFAULT_RECEIVER = "default-receiver",
  DEFAULT_SENDER = "default-sender",
  ANNOUNCEMENT = "announcement",
}

const MessageTypeMap = {
  [MessageType.DEFAULT_RECEIVER]: styles.messageWrapper,
  [MessageType.DEFAULT_SENDER]: `${styles.messageWrapper} ${styles.localSentMessage}`,
} as const;
export function getStyle(message: Message, localUser: TUser) {
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

type MessageSender = {
  id: string;
  name: string;
};

export type Message = {
  key: string;
  type: MessageType;
  sender: MessageSender;
  message: string;
};
