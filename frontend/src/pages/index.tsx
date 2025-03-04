import { useMemo, useState } from "react";
import {
  Button,
  Card,
  Code,
  Input,
  Listbox,
  ListboxItem,
  Pagination,
  Spinner,
  Table,
  TableBody,
  TableCell,
  TableColumn,
  TableHeader,
  TableRow,
  Tooltip,
} from "@heroui/react";
import Decimal from "decimal.js";
import dayjs from "dayjs";
import { useDebounceValue } from "usehooks-ts";
import { InfoIcon } from "lucide-react";

import { useAccountNames } from "@/api/accountnames";
import DefaultLayout from "@/layouts/default";
import { useAccountTransactions } from "@/api/accounttransactions";
import { useAccountBalances } from "@/api/accountbalances";

export default function IndexPage() {
  const queryAccountNames = useAccountNames();
  const [search, setSearch] = useState("");
  const [searchDebounce] = useDebounceValue(search, 1300, { trailing: true });
  const [selectedAccountNames, setSelectedAccountNames] = useState(
    new Set([] as string[]),
  );
  const items = useMemo<Array<{ text: string }>>(() => {
    if (!queryAccountNames.data) return [];

    return queryAccountNames.data.map((text) => ({ text }));
  }, [queryAccountNames]);

  const _setSelectedAccountNames = (s: Set<string>) => {
    setSelectedAccountNames(s);

    setSearch([...s].map((s) => s + "**").join("|"));
  };

  return (
    <DefaultLayout>
      <div className="flex justify-between w-full flex-wrap md:flex-nowrap gap-4">
        <Card className="w-full md:w-1/2 lg:w-1/3 p-2 gap-1 flex flex-col relative overflow-visible">
          <Input
            endContent={
              <Tooltip
                content={
                  <div className="px-1 py-2">
                    <div className="text-small font-bold mb-2">
                      Filter symbols
                    </div>
                    <div className="text-tiny">
                      <ul className="space-y-2">
                        <li>
                          <Code className="text-xs">*</Code> select any
                          character
                        </li>
                        <li>
                          <Code className="text-xs">**</Code> select zero or
                          more characters
                        </li>
                        <li>
                          <Code className="text-xs">|</Code> match either left
                          or right of it
                        </li>
                      </ul>
                    </div>
                  </div>
                }
                offset={15}
                placement="right-start"
              >
                <Button
                  isIconOnly
                  color="primary"
                  radius="md"
                  size="sm"
                  variant="light"
                >
                  <InfoIcon size={16} />
                </Button>
              </Tooltip>
            }
            value={search}
            onValueChange={setSearch}
          />
          <h1 className="mx-2 text-sm text-default-500 pt-1">
            Select one or many accounts:
          </h1>
          <Listbox
            isVirtualized
            className="w-full h-60"
            items={items}
            label={"Select from 1000 items"}
            selectedKeys={selectedAccountNames}
            selectionMode="multiple"
            virtualization={{
              maxListboxHeight: 400,
              itemHeight: 40,
            }}
            onSelectionChange={_setSelectedAccountNames as any}
          >
            {(item) => <ListboxItem key={item.text}>{item.text}</ListboxItem>}
          </Listbox>
        </Card>

        <div className="w-full md:w-1/2 lg:w-1/3">
          {searchDebounce ? (
            <BalanceTable selectedAccounts={searchDebounce} />
          ) : null}
        </div>
      </div>
      <section className="flex flex-col items-center justify-center gap-4 py-8 md:py-10">
        {searchDebounce ? (
          <TransactionsTable selectedAccounts={searchDebounce} />
        ) : null}
      </section>
    </DefaultLayout>
  );
}

function BalanceTable(props: { selectedAccounts: string }) {
  const { data: items, isLoading } = useAccountBalances(props.selectedAccounts);

  const columns = [
    { key: "account", label: "Account" },
    { key: "balance", label: "Balance" },
  ];

  if (isLoading) return <Spinner />;

  return (
    <Table aria-label="Example table with dynamic content">
      <TableHeader columns={columns}>
        {(column) => <TableColumn key={column.key}>{column.label}</TableColumn>}
      </TableHeader>
      <TableBody items={items}>
        {(item) => (
          <TableRow key={item.accountName + item.commodityUnit}>
            <TableCell>{item.accountName}</TableCell>
            <TableCell className="text-right font-mono">
              <Numberify amount={item.amount} t={item} />
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}
function TransactionsTable(props: { selectedAccounts: string }) {
  const { data: items, isLoading } = useAccountTransactions(
    props.selectedAccounts,
  );

  const columns = [
    { key: "date", label: "Date" },
    { key: "immutable_meta", label: "Immutable Metadata" },
    { key: "accounts", label: "Accounts In/out" },
    { key: "amount", label: "Amount" },
  ];

  const [page, setPage] = useState(1);
  const rowsPerPage = 20;

  const { paginatedItems, pages } = useMemo(() => {
    const start = (page - 1) * rowsPerPage;
    const end = start + rowsPerPage;

    const paginatedItems = items?.slice(start, end) || [];
    const pages = Math.ceil((items?.length || 0) / rowsPerPage);

    return { paginatedItems, pages };
  }, [page, items]);

  if (isLoading) return <Spinner />;

  return (
    <Table
      aria-label="Example table with dynamic content"
      bottomContent={
        <div
          className={"flex w-full justify-center".concat(
            pages > 1 ? "" : " hidden",
          )}
        >
          <Pagination
            isCompact
            showControls
            showShadow
            color="secondary"
            page={page}
            total={pages}
            onChange={(page) => setPage(page)}
          />
        </div>
      }
    >
      <TableHeader columns={columns}>
        {(column) => <TableColumn key={column.key}>{column.label}</TableColumn>}
      </TableHeader>
      <TableBody items={paginatedItems}>
        {(item) => (
          <TableRow key={item.transferId}>
            <TableCell>
              <dl className="inline-grid [&_dt]:col-start-1 [&_dt]:col-span-2 [&_dd]:col-start-3 [&_dd]:col-span-4 [&_dd]:font-mono text-right text-xs gap-1">
                <dt>Created timestamp:</dt>
                <dd title={item.fullDate.toString()}>
                  {dayjs.unix(item.fullDate).toISOString()}
                </dd>
                <dt>Custom timestamp:</dt>
                <dd title={item.fullDate2.toString()}>
                  {dayjs.unix(item.fullDate2).toISOString()}
                </dd>
              </dl>
            </TableCell>
            <TableCell>
              <dl className="inline-grid [&_dt]:col-start-1 [&_dt]:col-span-2 [&_dd]:col-start-3 [&_dd]:col-span-4 [&_dd]:font-mono text-right text-xs gap-1">
                <dt>Transfer ID:</dt>
                <dd>{item.transferId}</dd>
                <dt>Related ID:</dt>
                <dd>{item.relatedId}</dd>
                <dt>Code:</dt>
                <dd>{item.code}</dd>
              </dl>
            </TableCell>
            <TableCell>
              <ol>
                <li>{item.debitAccount}</li>
                <li>{item.creditAccount}</li>
              </ol>
            </TableCell>
            <TableCell>
              <ol className="text-right font-mono">
                <li className="text-success-700">
                  <Numberify amount={item.debitAmount} t={item} />
                </li>
                <li className="text-danger-700">
                  <Numberify amount={item.creditAmount} t={item} />
                </li>
              </ol>
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}

function Numberify(props: {
  t: { commodityDecimal: number; commodityUnit: string };
  amount: number;
}) {
  let n = Decimal(props.amount);

  if (props.t.commodityDecimal != 0) {
    n = n.div(Math.pow(10, props.t.commodityDecimal));
  }

  let str = n.toFixed(props.t.commodityDecimal);

  if (props.t.commodityUnit) {
    str = str + " " + props.t.commodityUnit;
  }

  return str;
}
