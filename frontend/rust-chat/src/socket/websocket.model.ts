import { TUser } from "../App.context";

export enum MessageTypes {
  GlobalChat = 0,
  MatchRequest = 1,
}

export type MessageType = GameRequestMessage | GlobalChatMessage;

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
