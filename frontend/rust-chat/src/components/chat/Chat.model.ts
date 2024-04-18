export enum MessageType {
  DEFAULT = "default",
}

type MessageSender = {
  id: string;
  name: string;
};

export type Message = {
  key: string;
  type: MessageType;
  sender: MessageSender;
  message: string;
};

export const FAKE_MESSAGES: Message[] = [
  {
    key: "alskdnalksdn65",
    type: MessageType.DEFAULT,
    sender: { id: "a", name: "cooldude124" },
    message: "Hello World",
  },
  {
    key: "kbnoknbnob24n",
    type: MessageType.DEFAULT,
    sender: { id: "b", name: "luv_crayons" },
    message: "This is a test!!!",
  },
  {
    key: "189239aff",
    type: MessageType.DEFAULT,
    sender: { id: "c", name: "Edgar Allen Poe" },
    message:
      "Lorem ipsum dolor sit, amet consectetur adipisicing elit. Voluptate, in aliquid sapiente id maiores, amet non dolorum ipsum dignissimos quibusdam dolor assumenda quod. Temporibus exercitationem praesentium enim, ducimus odio consequuntur?",
  },
  {
    key: "alskdnalksdn65",
    type: MessageType.DEFAULT,
    sender: { id: "a", name: "cooldude124" },
    message: "Hello World",
  },
  {
    key: "kbnoknbnob24n",
    type: MessageType.DEFAULT,
    sender: { id: "b", name: "luv_crayons" },
    message: "This is a test!!!",
  },
  {
    key: "189239aff",
    type: MessageType.DEFAULT,
    sender: { id: "c", name: "Edgar Allen Poe" },
    message:
      "Lorem ipsum dolor sit, amet consectetur adipisicing elit. Voluptate, in aliquid sapiente id maiores, amet non dolorum ipsum dignissimos quibusdam dolor assumenda quod. Temporibus exercitationem praesentium enim, ducimus odio consequuntur?",
  },
  {
    key: "alskdnalksdn65",
    type: MessageType.DEFAULT,
    sender: { id: "a", name: "cooldude124" },
    message: "1Hello World",
  },
  {
    key: "kbnoknbnob24n",
    type: MessageType.DEFAULT,
    sender: { id: "b", name: "luv_crayons" },
    message: "2This is a test!!!",
  },
  {
    key: "189239aff",
    type: MessageType.DEFAULT,
    sender: { id: "c", name: "Edgar Allen Poe" },
    message:
      "3Lorem ipsum dolor sit, amet consectetur adipisicing elit. Voluptate, in aliquid sapiente id maiores, amet non dolorum ipsum dignissimos quibusdam dolor assumenda quod. Temporibus exercitationem praesentium enim, ducimus odio consequuntur?",
  },
];
