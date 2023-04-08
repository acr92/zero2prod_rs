import Image from "next/image";
import { Inter } from "next/font/google";
import Navbar from "@/app/components/navbar";

const inter = Inter({ subsets: ["latin"] });

export const metadata = {
  title: "Zero2Prod - Sign-up",
};

export default function Home() {
  return (
    <div>
      <Navbar />

      <div className="hero min-h-screen bg-base-200">
        <div className="hero-content flex-col lg:flex-row-reverse">
          <div className="text-center lg:text-left">
            <h1 className="text-5xl font-bold">Sign-up!</h1>
            <p className="py-6">
              Sign up to our newsletter to get the latest updates
              on...something...
            </p>
          </div>
          <div className="card flex-shrink-0 w-full max-w-sm shadow-2xl bg-base-100">
            <div className="card-body">
              <div className="form-control">
                <label className="label">
                  <span className="label-text">Email</span>
                </label>
                <input
                  type="text"
                  placeholder="email"
                  className="input input-bordered"
                />
              </div>
              <div className="form-control mt-6">
                <button className="btn btn-primary">Sign up</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
