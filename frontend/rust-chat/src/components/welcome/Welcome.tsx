import React, { useContext, useRef } from "react";
import { AppContext } from "../../App.context";

export const Welcome: React.FC = () => {
  const { setLocalUser } = useContext(AppContext);
  const inputUsernameRef = useRef<HTMLInputElement>(null);
  return (
    <>
      <form
        onSubmit={(event: React.FormEvent<HTMLFormElement>) => {
          event.preventDefault();
          if (!inputUsernameRef.current) {
            throw Error("no input username ref");
          }
          inputUsernameRef.current.value =
            inputUsernameRef.current.value.trim();
          if (inputUsernameRef.current.value.length <= 0) {
            alert("username not acceptable");
            return;
          }
          setLocalUser({ id: "c", name: inputUsernameRef.current.value });
        }}
      >
        <label htmlFor="desired-username">desired username</label>
        <input
          id="desired-username"
          ref={inputUsernameRef}
        />
        <button>ok</button>
      </form>
    </>
  );
};
