import React, { useEffect, useRef, useState } from "react";
import { FAKE_MESSAGES, Message, MessageType } from "./Chat.model";
import styles from "./Chat.module.css";

export const Chat: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>(FAKE_MESSAGES);
  const [websocket, setWebsocket] = useState<WebSocket>();
  const messageInput = useRef<HTMLInputElement>(null);
  const chatWindow = useRef<HTMLDivElement>(null);
  const sendButton = useRef<HTMLButtonElement>(null);
  const localUserId = "c";
  const localUsername = "Edgar Allen Poe";

  // handle websocket connection
  useEffect(() => {
    const websocket = new WebSocket("ws://127.0.0.1:8080");
    websocket.onopen = (event: Event) => {
      console.log(event);
    };
    websocket.onmessage = (event: MessageEvent<any>) => {
      console.log(event);
      setMessages((messages) => {
        messages.push({
          key: "TBD",
          type: MessageType.DEFAULT,
          sender: {
            id: localUserId,
            name: localUsername,
          },
          message: event.data,
        });
        return [...messages];
      });
    };
    setWebsocket(websocket);
    return () => {
      websocket.close();
    };
  }, []);

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
    const messageToSend = messageInput.current.value.trim();
    if (messageToSend.length <= 0) {
      return;
    }
    if (websocket === undefined || websocket.readyState !== websocket.OPEN) {
      alert("Not connected to server");
      return;
    }
    websocket.send(
      JSON.stringify({
        key: "TBD",
        type: MessageType.DEFAULT,
        sender: {
          id: localUserId,
          name: localUsername,
        },
        message: messageToSend,
      })
    );
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
        {messages.map(({ message, type, sender, key }, index) => {
          const isLocalSender = localUserId === sender.id;
          return (
            <div
              key={`${index}-${key}`}
              className={cssClassConstructor([
                styles.message,
                isLocalSender ? styles.localSentMessage : "",
              ])}
            >
              <h2>{sender.name}</h2>
              {message.slice(1, -1)}
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

/**
 * TODO: move this somewhere
 * general utility function for generating class names
 * i just find using this is more legibile in jsx templates
 */
function cssClassConstructor(classes: string[]): string {
  return classes.join(" ").trim();
}
