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
});

export const AppContextProvider: React.FC<TAppContextProvider> = (props) => {
  const [localUser, setLocalUser] = useState<TUser | null>(null);
  const [messages, setMessages] = useState<GlobalChatMessage[]>([]);
  const [matchRequests, setMatchRequests] = useState<MatchRequest[]>([]);
  const [currentMatch, setCurrentMatch] = useState<GameMatch>();
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
      }}
    >
      {props.children}
    </AppContext.Provider>
  );
};
