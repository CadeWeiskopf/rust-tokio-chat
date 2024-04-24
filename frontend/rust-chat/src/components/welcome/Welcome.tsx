import React, { useContext, useRef, useState } from "react";
import { AppContext } from "../../App.context";

export const Welcome: React.FC = () => {
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const { setLocalUser } = useContext(AppContext);
  const inputUsernameRef = useRef<HTMLInputElement>(null);

  const handleRegister = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!inputUsernameRef.current) {
      throw Error("no input username ref");
    }
    inputUsernameRef.current.value = inputUsernameRef.current.value.trim();
    if (inputUsernameRef.current.value.length <= 0) {
      alert("username not acceptable");
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
    console.log("data=", id);
    setLocalUser({
      id,
      name: inputUsernameRef.current.value,
    });
  };

  return (
    <>
      <form onSubmit={handleRegister}>
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
