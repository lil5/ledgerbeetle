import { Button } from "@heroui/react";
import { GitForkIcon } from "lucide-react";

import { Navbar } from "@/components/navbar";
import { Link } from "react-router-dom";

export default function DefaultLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="relative flex flex-col h-screen">
      <Navbar />
      <main className="container mx-auto max-w-7xl px-4 flex-grow pt-4">
        {children}
      </main>
      <footer className="w-full flex items-center justify-center gap-4 px-4 py-3">
        <p className="text-default-500 text-sm">
          Lucian I. Last <span className="inline-block scale-[-1]">©</span>{" "}
          Apache 2.0
        </p>
        <Button
          as={Link}
          aria-label="Star lil5/ledgerbeetle on GitHub"
          className="font-semibold"
          size="sm"
          target="_blank"
          to="https://github.com/lil5/ledgerbeetle"
          variant="faded"
        >
          <GitForkIcon size={16} />
          Github
        </Button>
      </footer>
    </div>
  );
}
