import React, { ReactNode, useState } from "react";
import { Socket } from "./socket/websocket";
import { GlobalChatMessage } from "./socket/websocket.model";

export type TUser = {
  id: string;
  name: string;
};

export type MatchRequest = {
  requestFrom: TUser;
};

export type GameMatch = {
  users: TUser[];
};

export type GamePiece = {
  shape: "I" | "O" | "T" | "S" | "Z" | "J" | "L";
  position: { x: number; y: number };
};

interface IAppContext {
  localUser: TUser | null;
  setLocalUser: React.Dispatch<React.SetStateAction<TUser | null>>;
  sock: Socket;
  messages: GlobalChatMessage[];
  setMessages: React.Dispatch<React.SetStateAction<GlobalChatMessage[]>>;
  matchRequests: MatchRequest[];
  setMatchRequests: React.Dispatch<React.SetStateAction<MatchRequest[]>>;
  currentMatch?: GameMatch;
  setCurrentMatch: React.Dispatch<React.SetStateAction<GameMatch | undefined>>;
  gamePieces: GamePiece[];
  setGamePieces: React.Dispatch<React.SetStateAction<GamePiece[]>>;
}

type TAppContextProvider = {
  children: ReactNode[] | ReactNode;
};

const sock = new Socket();
export const AppContext = React.createContext<IAppContext>({
  localUser: null,
  setLocalUser: () => {},
  sock,
  messages: [],
  setMessages: () => {},
  matchRequests: [],
  setMatchRequests: () => {},
  setCurrentMatch: () => {},
  gamePieces: [],
  setGamePieces: () => undefined,
});

export const AppContextProvider: React.FC<TAppContextProvider> = (props) => {
  const [localUser, setLocalUser] = useState<TUser | null>(null);
  const [messages, setMessages] = useState<GlobalChatMessage[]>([]);
  const [matchRequests, setMatchRequests] = useState<MatchRequest[]>([]);
  const [currentMatch, setCurrentMatch] = useState<GameMatch>();
  const [gamePieces, setGamePieces] = useState<GamePiece[]>([]);
  return (
    <AppContext.Provider
      value={{
        localUser,
        setLocalUser,
        sock,
        messages,
        setMessages,
        matchRequests,
        setMatchRequests,
        currentMatch,
        setCurrentMatch,
        gamePieces,
        setGamePieces,
      }}
    >
      {props.children}
    </AppContext.Provider>
  );
};
