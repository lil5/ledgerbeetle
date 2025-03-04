import { Link } from "@heroui/link";
import {
  Navbar as HeroUINavbar,
  NavbarBrand,
  NavbarContent,
  NavbarItem,
} from "@heroui/navbar";

import { ThemeSwitch } from "./theme-switch";
import AddModal from "./add-modal";

export const Navbar = () => {
  return (
    <HeroUINavbar
      className=" bg-base-100/60 border-b-2 border-default-100"
      maxWidth="xl"
      position="sticky"
    >
      <NavbarContent className="flex-grow sm:basis-full" justify="start">
        <NavbarBrand className="gap-3 max-w-fit">
          <Link
            className="flex justify-start items-center gap-1"
            color="foreground"
            href="/"
          >
            <p className="font-bold text-inherit">LedgerBeetle</p>
          </Link>
        </NavbarBrand>
        <p className="text-xs">
          Combining the super powers of TigerBeetle
          <br />
          with the simplicity of hledger
        </p>
      </NavbarContent>

      <NavbarContent className="flex flex-grow-0" justify="end">
        <NavbarItem className="gap-2">
          <AddModal />
        </NavbarItem>
        <NavbarItem className="gap-2">
          {/* <Link isExternal href={siteConfig.links.github} title="GitHub">
            <GithubIcon className="text-default-500" />
          </Link> */}

          <ThemeSwitch />
        </NavbarItem>
      </NavbarContent>
    </HeroUINavbar>
  );
};
