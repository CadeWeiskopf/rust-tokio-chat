import React, { useRef, useState } from "react";
import { FAKE_MESSAGES, Message, MessageType } from "./Chat.model";
import styles from "./Chat.module.css";
import { sendMessage } from "../../api/messages";

const cssClassConstructor = (classes: string[]): string => {
  return classes.join(" ").trim();
};

export const Chat: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);
  const messageInput = useRef<HTMLInputElement>(null);
  const messages = FAKE_MESSAGES;
  const localUserId = "c";
  const localUsername = "Edgar Allen Poe";
  return (
    <div className={styles.wrapper}>
      {messages.map(({ message, type, sender, key }) => {
        const isLocalSender = localUserId === sender.id;
        return (
          <div
            className={cssClassConstructor([
              styles.message,
              isLocalSender ? styles.localSentMessage : "",
            ])}
          >
            {message}
          </div>
        );
      })}

      <div className={styles.inputWrapper}>
        <form
          onSubmit={async (event: React.FormEvent<HTMLFormElement>) => {
            event.preventDefault();
            setIsLoading(true);
            if (messageInput.current === null) {
              return;
            }
            const messageToSend = messageInput.current.value.trim();
            if (messageToSend.length <= 0) {
              return;
            }
            await sendMessage({
              key: "TBD",
              type: MessageType.DEFAULT,
              sender: {
                id: localUserId,
                name: localUsername,
              },
              message: messageToSend,
            });
            setIsLoading(false);
          }}
        >
          <input ref={messageInput} />
          <button disabled={isLoading}>
            {isLoading ? "loading..." : "send"}
          </button>
        </form>
      </div>
    </div>
  );
};
