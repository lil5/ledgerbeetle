import { useCallback, useMemo, useState } from "react";
import { Autocomplete, AutocompleteItem } from "@heroui/autocomplete";

import { useAccountNames } from "@/api/accountnames";
import DefaultLayout from "@/layouts/default";
import {
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
import { Pamount, useAccountTransactions } from "@/api/accounttransactions";
import Decimal from "decimal.js";

export default function IndexPage() {
  const queryAccountNames = useAccountNames();
  const [selectedAccountNames, setSelectedAccountNames] = useState(
    new Set([] as string[]),
  );
  const items = useMemo<Array<{ text: string }>>(() => {
    if (!queryAccountNames.data) return [];

    return queryAccountNames.data.map((text) => ({ text }));
  }, [queryAccountNames]);

  const selectedAccountNamesRe = useMemo<string | undefined>(() => {
    if (selectedAccountNames.size == 0) return undefined;

    return [...selectedAccountNames].join("|");
  }, [selectedAccountNames]);

  return (
    <DefaultLayout>
      <div className="flex w-full flex-wrap md:flex-nowrap gap-4">
        <Listbox
          isVirtualized
          className="max-w-xs"
          items={items}
          label={"Select from 1000 items"}
          selectedKeys={selectedAccountNames}
          selectionMode="multiple"
          virtualization={{
            maxListboxHeight: 400,
            itemHeight: 40,
          }}
          onSelectionChange={setSelectedAccountNames as any}
        >
          {(item) => <ListboxItem key={item.text}>{item.text}</ListboxItem>}
        </Listbox>
      </div>
      <section className="flex flex-col items-center justify-center gap-4 py-8 md:py-10">
        {selectedAccountNamesRe ? (
          <TransactionsTable selectedAccounts={selectedAccountNamesRe} />
        ) : null}
      </section>
    </DefaultLayout>
  );
}

function TransactionsTable(props: { selectedAccounts: string }) {
  const { data: items, isLoading } = useAccountTransactions(
    props.selectedAccounts,
  );

  const columns = [
    { key: "date", label: "Date" },
    { key: "description", label: "Description" },
    { key: "account", label: "Account" },
    { key: "amount", label: "Amount" },
  ];

  const Numberify = useCallback(
    (pamount: Pamount) => {
      let n = Decimal(pamount.aquantity.decimalMantissa);

      if (pamount.aquantity.decimalPlaces != 0) {
        n = n.divToInt(Decimal(pamount.aquantity.decimalPlaces));
      }

      let str = n.toFixed(pamount.aquantity.decimalPlaces);

      if (pamount.acommodity) {
        str = str + " " + pamount.astyle.asdecimalmark;
      }

      return str;
    },
    [items],
  );

  const [page, setPage] = useState(1);
  const rowsPerPage = 4;

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
        <div className="flex w-full justify-center">
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
          <TableRow key={item.tindex}>
            <TableCell>{item.tdate}</TableCell>
            <TableCell>
              <ol>
                {item.tpostings.map((v, i) => (
                  <li key={i}>{v.ptransaction_}</li>
                ))}
              </ol>
            </TableCell>
            <TableCell>
              <ol>
                {item.tpostings.map((v, i) => (
                  <li key={i}>{v.paccount}</li>
                ))}
              </ol>
            </TableCell>
            <TableCell>
              <ol className="text-right font-mono">
                {item.tpostings.map((v) => (
                  <li key={v.pcomment}>{Numberify(v.pamount.at(0)!)}</li>
                ))}
              </ol>
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}
