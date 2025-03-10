import { Link } from "@heroui/link";
import {
  Navbar as HeroUINavbar,
  NavbarBrand,
  NavbarContent,
  NavbarItem,
} from "@heroui/navbar";

import AddModal from "./add-modal";
import IncomeStatementModal from "./income-statement-modal";

const ALLOW_ADD = import.meta.env.VITE_ALLOW_ADD == "true";

export function Navbar() {
  return (
    <HeroUINavbar
      className=" bg-base-100/60 border-b-2 border-default-100"
      maxWidth="xl"
      position="sticky"
    >
      <NavbarContent className="flex-grow" justify="start">
        <NavbarBrand className="gap-3 max-w-fit">
          <Link
            className="flex justify-start items-center gap-1"
            color="foreground"
            href="/"
          >
            <p className="font-bold text-inherit">LedgerBeetle</p>
          </Link>
        </NavbarBrand>
        <p className="text-xs hidden md:block">
          Combining the super powers of TigerBeetle
          <br />
          with the simplicity of hledger
        </p>
      </NavbarContent>

      <NavbarContent className="flex !flex-grow-0" justify="end">
        <IncomeStatementModal />
        {ALLOW_ADD ? (
          <NavbarItem className="gap-2">
            <AddModal />
          </NavbarItem>
        ) : null}
      </NavbarContent>
    </HeroUINavbar>
  );
}
