import React, { useContext, useRef, useState } from "react";
import { AppContext } from "../../App.context";
import { MessageTypes } from "../../socket/websocket.model";

export const Welcome: React.FC = () => {
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const {
    setLocalUser,
    sock,
    setMessages,
    setMatchRequests,
    setCurrentMatch,
    setGamePieces,
  } = useContext(AppContext);
  const inputUsernameRef = useRef<HTMLInputElement>(null);

  const handleRegister = async (event: React.FormEvent<HTMLFormElement>) => {
    setIsLoading(true);
    event.preventDefault();
    if (!inputUsernameRef.current) {
      throw Error("no input username ref");
    }
    inputUsernameRef.current.value = inputUsernameRef.current.value.trim();
    try {
      if (inputUsernameRef.current.value.length <= 0) {
        alert("username not acceptable");
        setIsLoading(false);
        return;
      }
      const response = await fetch(`http://127.0.0.1:8081/id`, {
        method: "POST",
        body: JSON.stringify({
          username: inputUsernameRef.current.value,
        }),
      });
      if (!response.ok) {
        throw Error("response not ok");
      }
      const id = await response.text();
      console.debug(id);
      const connectSock = async () => {
        console.log("init");
        const newUser = {
          id,
          name: inputUsernameRef.current!.value,
        };
        await sock.init(newUser, () => {
          if (!sock.websocket) {
            throw Error("sock.websocket");
          }
          sock.websocket.onmessage = (event: MessageEvent<any>) => {
            console.debug(event);
            const message = JSON.parse(event.data);
            if (message.type === MessageTypes.GlobalChat) {
              setMessages((messages) => {
                messages.push(message);
                return [...messages];
              });
            } else if (message.type === MessageTypes.MatchRequest) {
              setMatchRequests((matchRequests) => {
                matchRequests.push(message);
                return [...matchRequests];
              });
            } else if (message.type === MessageTypes.MatchStart) {
              console.log("set match");
              setCurrentMatch({
                users: message.users,
              });
            } else if (message.type === MessageTypes.MatchUpdate) {
              setGamePieces(message.gamePieces);
            }
          };
          setLocalUser(newUser);
        });
      };
      connectSock();
    } catch (e) {
      console.error(e);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div>
      <h2>Enter lobby</h2>
      <form onSubmit={handleRegister}>
        <label htmlFor="desired-username">desired username</label>
        <input
          id="desired-username"
          ref={inputUsernameRef}
        />
        <button disabled={isLoading}>ok</button>
      </form>
    </div>
  );
};
