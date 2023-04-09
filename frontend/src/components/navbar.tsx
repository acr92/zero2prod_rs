// components/Navbar.tsx
import React from "react";
import Link from "next/link";
import { useRouter } from "next/router";
const Navbar = () => {
  const { pathname } = useRouter();

  const isInAdmin = pathname.startsWith("/admin");

  return (
    <div className="navbar bg-base-100">
      <div className="flex-1">
        <Link className="btn btn-ghost normal-case text-xl" href={"/"}>
          Zero2Prod
        </Link>
      </div>
      <div className="flex-none">
        <ul className="menu menu-horizontal px-1">
          <li>
            <Link href="/admin" className={isInAdmin ? "active" : ""}>
              Admin
            </Link>
          </li>
        </ul>
      </div>
    </div>
  );
};
export default Navbar;
