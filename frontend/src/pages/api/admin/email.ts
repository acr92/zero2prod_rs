import { getAccessToken, withApiAuthRequired } from "@auth0/nextjs-auth0";

export default withApiAuthRequired(async function products(req, res) {
  const { accessToken } = await getAccessToken(req, res, {
    scopes: ["openid", "profile", "email"],
  });
  const response = await fetch("http://127.0.0.1:8000/admin/me", {
    headers: {
      Authorization: `Bearer ${accessToken}`,
    },
  });

  res.status(response.status).send(await response.text());
});
