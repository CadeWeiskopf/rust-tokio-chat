import React, { useContext, useEffect, useRef } from "react";
import { getStyle } from "./Chat.model";
import styles from "./Chat.module.css";
import { AppContext } from "../../App.context";
import { MessageTypes } from "../../socket/websocket.model";

export const Chat: React.FC = () => {
  const messageInput = useRef<HTMLInputElement>(null);
  const chatWindow = useRef<HTMLDivElement>(null);
  const sendButton = useRef<HTMLButtonElement>(null);
  const { localUser, sock, messages } = useContext(AppContext);

  if (localUser === null) {
    throw Error("user cannot be null to use chat");
  }

  // when message received scroll the chat window down
  useEffect(() => {
    chatWindow.current?.scrollTo({
      top: chatWindow.current?.scrollHeight,
      behavior: "smooth",
    });
  }, [messages]);

  // function on chat form submit to send the message
  const handleSendMessage = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!messageInput.current) {
      throw Error("missing message input ref");
    }
    const message = messageInput.current.value.trim();
    if (message.length <= 0) {
      return;
    }

    sock.send({
      type: MessageTypes.GlobalChat,
      sender: localUser,
      message,
    });

    messageInput.current.value = "";
    messageInput.current.dispatchEvent(
      new Event("input", { bubbles: true, cancelable: true })
    );
    messageInput.current.focus();
  };

  return (
    <div className={styles.wrapper}>
      <div
        className={styles.messageWrapper}
        ref={chatWindow}
      >
        {/* render the messages */}
        {messages.map((message, index) => {
          return (
            <div className={styles.messageBorder}>
              <div
                key={`${index}-${message.key}`}
                className={getStyle(message, localUser)}
              >
                <h2>{message.sender.name}</h2>
                {message.message}
              </div>
            </div>
          );
        })}
      </div>

      {/* chat message input */}
      <div className={styles.inputWrapper}>
        <form onSubmit={handleSendMessage}>
          <input
            ref={messageInput}
            onInput={(event: React.FormEvent<HTMLInputElement>) => {
              if (!sendButton.current) {
                throw Error("missing send button ref");
              }
              sendButton.current.disabled =
                event.currentTarget.value.trim().length <= 0;
            }}
          />
          <button
            ref={sendButton}
            disabled={true}
          >
            send
          </button>
        </form>
      </div>
    </div>
  );
};
