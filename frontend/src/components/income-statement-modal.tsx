/* eslint-disable no-console */
import {
  Button,
  Modal,
  ModalBody,
  ModalContent,
  ModalHeader,
  useDisclosure,
  DateInput,
  CardFooter,
  Card,
  CardBody,
  Table,
  TableHeader,
  TableColumn,
  TableBody,
  TableRow,
  TableCell,
  ModalFooter,
} from "@heroui/react";
import { fromDate, ZonedDateTime } from "@internationalized/date";
import { PlusIcon } from "lucide-react";
import { useRef, useState } from "react";
import dayjs from "dayjs";
import { useStore } from "@tanstack/react-store";

import Numberify from "./numberify";

import { useAccountIncomeStatements } from "@/api/accountincomestatements";
import { selectedAccountsStore } from "@/stores/selected-accounts-store";

export default function IncomeStatementModal() {
  const { isOpen, onOpen, onOpenChange } = useDisclosure();

  return (
    <>
      <Button variant="solid" onPress={onOpen}>
        <span>
          <span className="sm:hidden">IS</span>
          <span className="hidden lg:inline">Generate </span>
          <span className="hidden sm:inline">Income Statement</span>
        </span>
      </Button>

      <Modal
        isOpen={isOpen}
        scrollBehavior="outside"
        size="4xl"
        onOpenChange={onOpenChange}
      >
        <ModalContent>
          {() => (
            <>
              <ModalHeader className="flex flex-col gap-1">
                Generate Income Statement
              </ModalHeader>
              <IncomeStatementForm />
            </>
          )}
        </ModalContent>
      </Modal>
    </>
  );
}

function IncomeStatementForm() {
  const account_re = useStore(selectedAccountsStore);
  const elDatesScroll = useRef<HTMLDivElement>(null);
  const [dates, setDates] = useState(() => [
    { key: crypto.randomUUID(), d: dayjs().valueOf() },
    { key: crypto.randomUUID(), d: dayjs().subtract(7, "days").valueOf() },
  ]);
  const setZonedDate = (key: string, zd: ZonedDateTime) => {
    setDates((s) =>
      s.map((item) => {
        if (item.key !== key) return item;

        return { ...item, d: zd.toDate().valueOf() };
      }),
    );
  };
  const addDate = () => {
    setDates((s) => [
      ...s,
      {
        key: crypto.randomUUID(),
        d: dayjs(s.at(-1)?.d).subtract(7, "days").valueOf(),
      },
    ]);
    setTimeout(() => {
      elDatesScroll.current?.scrollTo({
        left:
          elDatesScroll.current.querySelector("&>div")?.getBoundingClientRect()
            .width || 0,
        behavior: "smooth",
      });
    }, 100);
  };
  const removeDate = (key: string) =>
    setDates((s) => s.filter((v) => v.key !== key));

  const { data } = useAccountIncomeStatements(
    account_re,
    dates.map((d) => d.d),
  );

  const columns = [
    { i: 0, d: 0 },
    ...data.dates.map((d, i) => ({ i: i + 1, d })),
  ];

  return (
    <>
      <ModalBody className="w-full">
        <div ref={elDatesScroll} className="overflow-x-auto w-full">
          <div className="inline-flex flex-row min-h-36 gap-2 m-2">
            {dates.map((d, i) => (
              <Card key={d.key} className="w-60">
                <CardBody className="gap-2">
                  <p className="text-sm">Date {i + 1}</p>
                  <DateInput
                    label="Date"
                    name={"date" + i}
                    value={fromDate(new Date(d.d), "Europe/Amsterdam")}
                    onChange={(zd) => setZonedDate(d.key, zd!)}
                  />
                </CardBody>
                <CardFooter className="justify-end">
                  <Button
                    color="danger"
                    size="sm"
                    onPress={() => removeDate(d.key)}
                  >
                    Remove
                  </Button>
                </CardFooter>
              </Card>
            ))}
            <div className="w-10 self-stretch flex items-center overflow-hidden">
              <div className="w-10 h-10">
                <Button
                  key="add-posting"
                  className="-translate-x-[calc(50%-20px)] rotate-90"
                  color="secondary"
                  variant="flat"
                  onPress={addDate}
                >
                  Add Posting
                  <PlusIcon className="ms-0.5" />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </ModalBody>
      <ModalFooter>
        <Table
          isStriped
          aria-label="Example table with dynamic content"
          classNames={{ wrapper: "p-0" }}
          radius="none"
          shadow="none"
        >
          <TableHeader columns={columns}>
            {(column) => {
              if (column.i === 0) {
                return <TableColumn key="account">Account Name</TableColumn>;
              }

              return (
                <TableColumn key={column.i}>
                  {dayjs(column.d).toISOString()}
                </TableColumn>
              );
            }}
          </TableHeader>
          <TableBody items={data.incomeStatements}>
            {(item) => (
              <TableRow key={item.accountName + item.commodityUnit}>
                {columns.map((_, i) => {
                  if (i === 0) {
                    return <TableCell key={i}>{item.accountName}</TableCell>;
                  }
                  const amount = item.amounts[i - 1];

                  return (
                    <TableCell
                      key={item.accountName + i}
                      className="text-right"
                    >
                      <Numberify amount={amount} t={item} />
                    </TableCell>
                  );
                })}
              </TableRow>
            )}
          </TableBody>
        </Table>
      </ModalFooter>
    </>
  );
}
