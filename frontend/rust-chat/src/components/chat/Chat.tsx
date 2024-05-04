import React, { useContext, useEffect, useRef, useState } from "react";
import { Message, MessageType, getStyle } from "./Chat.model";
import styles from "./Chat.module.css";
import { AppContext } from "../../App.context";

export const Chat: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [websocket, setWebsocket] = useState<WebSocket>();
  const messageInput = useRef<HTMLInputElement>(null);
  const chatWindow = useRef<HTMLDivElement>(null);
  const sendButton = useRef<HTMLButtonElement>(null);
  const { localUser } = useContext(AppContext);
  if (localUser === null) {
    throw Error("user cannot be null to use chat");
  }

  // handle websocket connection
  useEffect(() => {
    const websocket = new WebSocket(
      `ws://127.0.0.1:8080?username=${localUser.name}&id=${localUser.id}`
    );
    websocket.onopen = (event: Event) => {
      console.log(event);
    };
    websocket.onmessage = (event: MessageEvent<any>) => {
      console.log(event);
      // TODO: validate this data
      const message = JSON.parse(event.data);
      setMessages((messages) => {
        messages.push(message);
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
        sender: localUser,
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
