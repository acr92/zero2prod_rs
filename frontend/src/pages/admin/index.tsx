import { Inter } from "next/font/google";
import Navbar from "@/components/navbar";
import Head from "next/head";
import { useUser } from "@auth0/nextjs-auth0/client";

const inter = Inter({ subsets: ["latin"] });

export default function Home() {
  const { user, error, isLoading } = useUser();

  const userInfo = isLoading ? (
    <div>Loading...</div>
  ) : user ? (
    <div>userInfo: {JSON.stringify(user, null)}</div>
  ) : (
    <div>error: {error?.message}</div>
  );

  return (
    <div className={"flex flex-col h-screen"}>
      <Head>
        <title>Zero2Prod - Admin</title>
      </Head>

      <div>
        <Navbar />
      </div>
      <div className="hero h-screen bg-base-200">
        <div className="hero-content flex-col lg:flex-row-reverse">
          <div className="text-center lg:text-left">
            <h1 className="text-5xl font-bold">Hello Admin</h1>
            <p className="py-6">This will be our admin panel</p>
            {userInfo}
          </div>
        </div>
      </div>
    </div>
  );
}
