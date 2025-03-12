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
  Checkbox,
  Textarea,
} from "@heroui/react";
import { now, parseZonedDateTime } from "@internationalized/date";
import { PlusIcon, Trash2Icon } from "lucide-react";
import { useMemo, useState } from "react";
import { z } from "zod";
import { fromError } from "zod-validation-error";

import { useAddTransactions, useAddTransactionsGlob } from "@/api/add";
import { useAccountNames } from "@/api/accountnames";
import {
  AddFilterTransaction,
  AddFilterTransactions,
  AddTransaction,
  AddTransactions,
} from "@/client";

const RE_IS_ACCOUNT = /^(a|l|e|r|x):.*/;
const RE_HEXADECIMAL = /^[a-f0-9]{1,31}$/;

const zodAddTransaction = z.object({
  fullDate2: z.number(),
  transactions: z
    .array(
      z.object({
        code: z.number().min(1),
        commodityUnit: z.string().min(1),
        relatedId: z
          .string()
          .regex(
            RE_HEXADECIMAL,
            "must a length between 1 - 31 and compose of a-f and/or 0-9 characters",
          ),
        debitAccount: z.string().regex(RE_IS_ACCOUNT),
        creditAccount: z.string().regex(RE_IS_ACCOUNT),
        amount: z.number(),
      }),
    )
    .min(1),
});
const zodAddFilterTransaction = z.object({
  fullDate2: z.number(),
  transactions: z
    .array(
      z.object({
        code: z.number().min(1),
        commodityUnit: z.string().min(1),
        relatedId: z
          .string()
          .regex(
            RE_HEXADECIMAL,
            "must a length between 1 - 31 and compose of a-f and/or 0-9 characters",
          ),
        debitAccount: z.string().regex(RE_IS_ACCOUNT),
        creditAccountsFilter: z.array(z.string().regex(RE_IS_ACCOUNT)).min(1),
        amount: z.number(),
      }),
    )
    .min(1),
});

export default function AddModal() {
  const mutationAddGlob = useAddTransactionsGlob();
  const mutationAdd = useAddTransactions();
  const [isGlob, setIsGlob] = useState(false);
  const { isOpen, onOpen, onOpenChange } = useDisclosure();
  const [alertErr, setAlertErr] = useState("");
  const [submitLoading, setSubmitLoading] = useState(false);

  const [postings, setPostings] = useState(() => [
    { key: crypto.randomUUID() },
  ]);
  const addPosting = () =>
    setPostings((s) => [...s, { key: crypto.randomUUID() }]);
  const removePosting = (key: string) =>
    setPostings((s) => s.filter((v) => v.key !== key));

  function onSubmit(e: any) {
    e.preventDefault();
    setSubmitLoading(true);

    (async () => {
      const data: Record<string, any> = Object.fromEntries(
        new FormData(e.target),
      );

      const date = parseZonedDateTime(
        (
          (e.target as HTMLElement).querySelector(
            "[name=date]",
          )! as HTMLInputElement
        ).value,
      )
        .toDate()
        .valueOf();

      const transactions: Array<AddTransaction | AddFilterTransaction> = [];
      let i = 0;

      while (true) {
        const debit = data["debit" + i];

        if (!debit) break;

        const amount = data["amount" + i]!;
        const unit = data["unit" + i]!;
        const code = data["code" + i]!;
        const related_id = data["related_id" + i]!;

        if (isGlob) {
          const creditAccountsFilter: string[] = [];

          let ii = 0;

          while (true) {
            const credit = data["credit" + i + "-" + ii]!;

            if (!credit) break;

            creditAccountsFilter.push(credit);
            ii++;
          }

          transactions.push({
            code: parseInt(code),
            commodityUnit: unit,
            relatedId: related_id,
            debitAccount: debit,
            creditAccountsFilter,
            amount: parseInt(amount),
          });
        } else {
          const credit = data["credit" + i]!;

          transactions.push({
            code: parseInt(code),
            commodityUnit: unit,
            relatedId: related_id,
            debitAccount: debit,
            creditAccount: credit,
            amount: parseInt(amount),
          });
        }

        i++;
      }

      const addTransactionsData = {
        fullDate2: date,
        transactions,
      };

      console.info("add transaction", addTransactionsData);

      try {
        if (isGlob) {
          zodAddFilterTransaction.parse(addTransactionsData);
        } else {
          zodAddTransaction.parse(addTransactionsData);
        }
      } catch (err) {
        const validationError = fromError(err);

        console.warn("Validation error:", validationError);
        setAlertErr(validationError.toString());

        throw validationError;
      }
      try {
        if (isGlob) {
          await mutationAddGlob.mutateAsync({
            filterTransactions:
              addTransactionsData.transactions as AddFilterTransaction[],
            fullDate2: addTransactionsData.fullDate2,
          });
        } else {
          await mutationAdd.mutateAsync(addTransactionsData as AddTransactions);
        }
      } catch (err: any) {
        console.error("Mutation async add transaction error:", err);
        setAlertErr((err || "Unknown error").toString());

        throw err;
      }
      setAlertErr("");
    })().then(
      () => {
        setTimeout(() => {
          setPostings([]);
          setSubmitLoading(false);
        }, 1300);
      },
      () => {
        setTimeout(() => {
          setSubmitLoading(false);
        }, 1300);
      },
    );
  }

  return (
    <>
      <Button color="success" variant="bordered" onPress={onOpen}>
        <span className="text-foreground">
          Add<span className="hidden sm:inline"> Transactions</span>
        </span>
        <PlusIcon className="ms-1" />
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
                  label="Custom date"
                  name="date"
                />
                <div>
                  <Checkbox checked={isGlob} onValueChange={setIsGlob}>
                    Pool credit accounts from a search list
                  </Checkbox>
                  <p className="text-default-500 text-sm mt-0.5">
                    This will require the sum balance of all found
                    credit-accounts to have enough to facilitate each
                    transaction.
                  </p>
                </div>

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
                        {isGlob ? (
                          <AccountNamesGlob name={"credit" + i} />
                        ) : (
                          <AutocompleteAccountNames
                            className="col-span-3"
                            label="Credit account"
                            name={"credit" + i}
                          />
                        )}
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
                        <p className="text-sm">Metadata</p>
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
                <Button color="primary" isLoading={submitLoading} type="submit">
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

const DEFAULT_ACCOUNT_NAMES = ["a", "l", "e", "r", "x"];

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

function AccountNamesGlob(props: { name: string }) {
  const [list, setList] = useState([
    {
      key: crypto.randomUUID(),
    },
  ]);
  const addList = () => setList((s) => [...s, { key: crypto.randomUUID() }]);
  const removeList = (key: string) =>
    setList((s) => s.filter((v) => v.key !== key));

  return (
    <div className="space-y-2 col-span-full">
      <p className="text-xs text-default-700">
        Credit account search ordered by first come first served
      </p>
      {list.map((item, i) => (
        <Card key={item.key} className="p-2 flex flex-row gap-2 w-full">
          <p className="font-mono ms-1">{i + 1}</p>
          <Textarea
            className="flex-grow"
            name={props.name + "-" + i}
            placeholder="a:b*nk:**"
          />
          <Button
            isIconOnly
            color="danger"
            size="sm"
            onPress={() => removeList(item.key)}
          >
            <Trash2Icon size={20} />
          </Button>
        </Card>
      ))}
      <Button key="add" fullWidth onPress={addList}>
        Add credit account <PlusIcon />
      </Button>
    </div>
  );
}
