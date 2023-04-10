import { handleAuth, handleLogin } from "@auth0/nextjs-auth0";

export default handleAuth({
  login: handleLogin({
    authorizationParams: {
      audience: "http://127.0.0.1:8000",
      scope: "openid profile email send:emails",
    },
  }),
});
