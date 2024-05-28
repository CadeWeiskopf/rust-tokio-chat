import React, { useContext, useEffect, useState } from "react";
import { Chat } from "../../components/chat/Chat";
import { AppContext } from "../../App.context";
import { MessageTypes } from "../../socket/websocket.model";

export const Lobby: React.FC = () => {
  const challengeUserIdInputId = "challenge-user-id";
  const { sock, localUser, matchRequests } = useContext(AppContext);

  if (localUser === null) {
    throw Error("localUser cannot be null in lobby");
  }

  const handleChallengeSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!sock.websocket) {
      throw Error("no websocket");
    }
    const userToChallenge = (
      document.getElementById(challengeUserIdInputId) as HTMLInputElement
    ).value.trim();
    sock.send({
      type: MessageTypes.MatchRequest,
      matchRequest: userToChallenge,
    });
    alert(`challenge sent to ${userToChallenge}`);
  };

  return (
    <div>
      <h1>lobby</h1>
      <div>
        <h2>start a game</h2>
        <form onSubmit={handleChallengeSubmit}>
          <label htmlFor={challengeUserIdInputId}>challenge user:</label>
          <input id={challengeUserIdInputId} />
          <button>challenge</button>
        </form>
      </div>
      <div>
        <h2>chat</h2>
        {sock.websocket && <Chat />}
      </div>
      <div>
        {matchRequests.map((matchRequest) => (
          <>{JSON.stringify(matchRequest)}</>
        ))}
      </div>
    </div>
  );
};
