import React, { ReactNode, useState } from "react";

type TUser = {
  id: string;
  name: string;
};

interface IAppContext {
  localUser: TUser | null;
  setLocalUser: React.Dispatch<React.SetStateAction<TUser | null>>;
}

type TAppContextProvider = {
  children: ReactNode[] | ReactNode;
};

export const AppContext = React.createContext<IAppContext>({
  localUser: null,
  setLocalUser: () => {},
});

export const AppContextProvider: React.FC<TAppContextProvider> = (props) => {
  const [localUser, setLocalUser] = useState<TUser | null>(null);
  return (
    <AppContext.Provider value={{ localUser, setLocalUser }}>
      {props.children}
    </AppContext.Provider>
  );
};
