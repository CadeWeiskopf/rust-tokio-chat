import { Message } from "../components/chat/Chat.model";

export const sendMessage = async (message: Message) => {
  await new Promise<void>((resolve) => setTimeout(() => resolve(), 2000));
  console.log("sent", message);
};
