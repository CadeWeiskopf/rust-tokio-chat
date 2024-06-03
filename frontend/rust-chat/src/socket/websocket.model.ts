import { TUser } from "../App.context";

export enum MessageTypes {
  GlobalChat = 0,
  MatchRequest = 1,
  MatchAccept = 2,
  MatchStart = 3,
  MatchUpdate = 4,
}

export type MessageType =
  | GameRequestMessage
  | GlobalChatMessage
  | GameAcceptMessage;

export type GameRequestMessage = {
  type: MessageTypes.MatchRequest;
  matchRequest: string;
};

export type GlobalChatMessage = {
  type: MessageTypes.GlobalChat;
  key?: string;
  sender: TUser;
  message: string;
};

export type GameAcceptMessage = {
  type: MessageTypes.MatchAccept;
  matchAccept: string;
  sender: TUser;
};

export type GameStartMessage = {
  type: MessageTypes.MatchStart;
  users: TUser[];
};
