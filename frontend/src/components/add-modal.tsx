/* eslint-disable no-console */
import {
  Button,
  Modal,
  ModalBody,
  ModalContent,
  ModalFooter,
  Form,
  ModalHeader,
  useDisclosure,
  DateInput,
  CardFooter,
  Card,
  CardBody,
  Autocomplete,
  AutocompleteItem,
  Input,
  Alert,
} from "@heroui/react";
import { now, parseZonedDateTime } from "@internationalized/date";
import { PlusIcon } from "lucide-react";
import { useMemo, useState } from "react";
import { z } from "zod";
import { fromError } from "zod-validation-error";
import dayjs from "dayjs";

import { AddTransaction, useAddTransactions } from "@/api/add";
import { useAccountNames } from "@/api/accountnames";

const IS_ACCOUNT_RE = /^(assets|liabilities|equity|revenues|expenses):.*/;

const zodAddTransaction = z.object({
  fullDate2: z.number(),
  transactions: z
    .array(
      z.object({
        code: z.number().min(1),
        commodityUnit: z.string().min(1),
        relatedId: z.string().min(1),
        debitAccount: z.string().regex(IS_ACCOUNT_RE),
        creditAccount: z.string(),
        amount: z.number(),
      }),
    )
    .min(1),
});

export default function AddModal() {
  const mutationAdd = useAddTransactions();

  const { isOpen, onOpen, onOpenChange } = useDisclosure();
  const [alertErr, setAlertErr] = useState("");

  const [postings, setPostings] = useState(() => [
    { key: crypto.randomUUID() },
  ]);
  const addPosting = () =>
    setPostings((s) => [...s, { key: crypto.randomUUID() }]);
  const removePosting = (key: string) =>
    setPostings((s) => s.filter((v) => v.key !== key));

  function onSubmit(e: any) {
    e.preventDefault();
    const data: Record<string, any> = Object.fromEntries(
      new FormData(e.target),
    );

    const date = dayjs(
      parseZonedDateTime(
        (
          (e.target as HTMLElement).querySelector(
            "[name=date]",
          )! as HTMLInputElement
        ).value,
      ).toDate(),
    ).unix();

    const transactions: AddTransaction[] = [];
    let i = 0;

    while (true) {
      const debit = data["debit" + i];

      if (!debit) break;

      const credit = data["credit" + i]!;
      const amount = data["amount" + i]!;
      const unit = data["unit" + i]!;
      const code = data["code" + i]!;
      const related_id = data["related_id" + i]!;

      transactions.push({
        code: parseInt(code),
        commodityUnit: unit,
        relatedId: related_id,
        debitAccount: debit,
        creditAccount: credit,
        amount: parseInt(amount),
      });

      i++;
    }

    const addTransactionsData = {
      fullDate2: date,
      transactions,
    };

    try {
      zodAddTransaction.parse(addTransactionsData);
    } catch (err) {
      const validationError = fromError(err);

      console.warn("Validation error:", validationError);
      setAlertErr(validationError.toString());

      return;
    }
    try {
      mutationAdd.mutateAsync(addTransactionsData);
    } catch (err) {
      console.error("Mutation async add transaction error:", err);
      setAlertErr((err || "Unknown error").toString());

      return;
    }
    setAlertErr("");
  }

  return (
    <>
      <Button color="success" variant="bordered" onPress={onOpen}>
        Add Transactions <PlusIcon className="ms-1" />
      </Button>
      <Modal
        isOpen={isOpen}
        scrollBehavior="outside"
        size="xl"
        onOpenChange={onOpenChange}
      >
        <ModalContent>
          {(onClose) => (
            <Form className="flex flex-col w-full" onSubmit={onSubmit}>
              <ModalHeader className="flex flex-col gap-1">
                Add Transactions
              </ModalHeader>
              <ModalBody className="w-full">
                <DateInput
                  defaultValue={now("Europe/Amsterdam")}
                  label="Date"
                  name="date"
                />

                {postings.map((post, i) => (
                  <Card key={post.key}>
                    <CardBody className="gap-2">
                      <p className="text-sm">Transaction {i + 1}</p>
                      <div className="gap-2 grid grid-cols-6">
                        <AutocompleteAccountNames
                          className="col-span-3"
                          label="Debit account"
                          name={"debit" + i}
                        />
                        <AutocompleteAccountNames
                          className="col-span-3"
                          label="Credit account"
                          name={"credit" + i}
                        />
                        <Input
                          className="row-start-1 col-start-4 col-span-2"
                          defaultValue="0"
                          label="Amount"
                          name={"amount" + i}
                          type="number"
                        />
                        <Input
                          className="row-start-1 col-start-6 col-span-1"
                          defaultValue="€"
                          label="Unit"
                          name={"unit" + i}
                          type="string"
                        />
                      </div>
                      <div className="flex flex-col gap-2">
                        <p className="text-sm">Meta data</p>
                        <Input
                          label="Code"
                          name={"code" + i}
                          size="sm"
                          type="number"
                        />
                        <Input
                          label="Related ID"
                          name={"related_id" + i}
                          size="sm"
                          type="text"
                        />
                      </div>
                    </CardBody>
                    <CardFooter className="justify-end">
                      <Button
                        color="danger"
                        size="sm"
                        onPress={() => removePosting(post.key)}
                      >
                        Remove
                      </Button>
                    </CardFooter>
                  </Card>
                ))}

                <Button onPress={addPosting}>
                  Add Posting
                  <PlusIcon className="ms-0.5" />
                </Button>

                {alertErr ? <Alert color="danger" title={alertErr} /> : null}
              </ModalBody>
              <ModalFooter className="w-full flex-row justify-between">
                <Button color="danger" variant="light" onPress={onClose}>
                  Close
                </Button>
                <Button color="primary" type="submit">
                  Submit
                </Button>
              </ModalFooter>
            </Form>
          )}
        </ModalContent>
      </Modal>
    </>
  );
}

const DEFAULT_ACCOUNT_NAMES = [
  "assets",
  "liabilities",
  "equity",
  "revenues",
  "expenses",
];

function AutocompleteAccountNames(props: {
  name: string;
  label: string;
  className?: string;
}) {
  const queryAccountNames = useAccountNames();
  const items = useMemo(() => {
    const list = queryAccountNames.data ? [...queryAccountNames.data] : [];

    DEFAULT_ACCOUNT_NAMES.forEach((name) => {
      if (!list.includes(name)) {
        list.push(name);
      }
    });

    return list
      .map((name) => ({ name }))
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [queryAccountNames]);

  return (
    <Autocomplete
      allowsCustomValue
      className={props.className}
      defaultItems={items}
      label={props.label}
      name={props.name}
      placeholder="Search an account"
    >
      {(item) => (
        <AutocompleteItem key={item.name}>{item.name}</AutocompleteItem>
      )}
    </Autocomplete>
  );
}
