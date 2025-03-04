import { useMemo, useState } from "react";
import {
  Card,
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
} from "@heroui/react";
import Decimal from "decimal.js";
import dayjs from "dayjs";
import { useDebounceValue } from "usehooks-ts";

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
      <div className="flex  justify-between w-full flex-wrap md:flex-nowrap gap-4">
        <Card className="w-1/2 sm:w-1/3 p-2">
          <Input value={search} onValueChange={setSearch} />
          <h1 className="mx-2">Select one or many accounts</h1>
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

        <div>
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
    { key: "description", label: "Description" },
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
            <TableCell>{dayjs.unix(item.fullDate).format()}</TableCell>
            <TableCell>
              <dl className="grid [&_dt]:col-start-1 [&_dt]:col-span-2 [&_dd]:col-start-3 [&_dd]:col-span-4 [&_dd]:font-mono text-right text-xs gap-1">
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
                {[item.debitAmount, item.creditAmount].map((v, i) => (
                  <li key={i}>
                    <Numberify amount={v} t={item} />
                  </li>
                ))}
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
