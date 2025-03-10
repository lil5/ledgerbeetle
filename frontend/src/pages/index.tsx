import { useMemo, useState } from "react";
import {
  Button,
  Card,
  CardBody,
  CardHeader,
  Checkbox,
  Code,
  DatePicker,
  DateRangePicker,
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
import { now } from "@internationalized/date";
import dayjs from "dayjs";
import { useDebounceValue } from "usehooks-ts";
import { InfoIcon } from "lucide-react";

import { useAccountNames } from "@/api/accountnames";
import DefaultLayout from "@/layouts/default";
import { useAccountTransactions } from "@/api/accounttransactions";
import { useAccountBalances } from "@/api/accountbalances";
import useStoreHook from "@/stores/hook";
import { selectedAccountsStore } from "@/stores/selected-accounts-store";
import Numberify from "@/components/numberify";

export default function IndexPage() {
  const queryAccountNames = useAccountNames();
  const [isOpenTooltip, setIsOpenTooltip] = useState(false);
  const [search, setSearch] = useStoreHook(selectedAccountsStore);
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

    setSearch(() => [...s].map((s) => s + "**").join("|"));
  };

  return (
    <DefaultLayout>
      <div className="flex justify-between w-full flex-wrap md:flex-nowrap gap-4">
        <div className="w-full md:w-1/2 lg:w-1/3">
          <Card className="p-2 gap-1 flex flex-col relative overflow-visible">
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
                  isOpen={isOpenTooltip}
                  offset={15}
                  placement="right-start"
                >
                  <Button
                    isIconOnly
                    color="primary"
                    radius="md"
                    size="sm"
                    variant="light"
                    onBlur={() => setIsOpenTooltip(false)}
                    onFocus={() => setIsOpenTooltip(true)}
                    onMouseEnter={() => setIsOpenTooltip(true)}
                    onMouseLeave={() => setIsOpenTooltip(false)}
                  >
                    <InfoIcon size={16} />
                  </Button>
                </Tooltip>
              }
              value={search}
              onValueChange={(s) => setSearch(() => s)}
            />
            <h1 className="mx-2 text-sm text-default-500 pt-1">
              Select one or many accounts:
            </h1>
            <Listbox
              isVirtualized
              classNames={{ list: "w-full h-60" }}
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
        </div>

        <div className="w-full md:w-1/2 lg:w-1/3">
          <BalanceTable selectedAccounts={searchDebounce} />
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
  const [showFilter, setShowFilter] = useState(false);
  const [filterDate, setFilterDate] = useState(now("Europe/Amsterdam"));
  const filterDateIfTrue = useMemo(() => {
    if (showFilter) return filterDate.toDate().valueOf();

    return null;
  }, [showFilter, filterDate]);
  const { data: items, isFetching } = useAccountBalances(
    props.selectedAccounts,
    filterDateIfTrue,
  );

  const columns = [
    { key: "account", label: "Account" },
    { key: "balance", label: "Balance" },
  ];

  return (
    <Card>
      <CardHeader className="gap-2 pb-0">
        <h2 className="font-bold text-default-500 flex-grow">Balance</h2>
        <Spinner className={isFetching ? "" : "hidden"} size="sm" />
        <Checkbox
          classNames={{ base: "flex-row-reverse", wrapper: "me-0 ms-2" }}
          color="secondary"
          isSelected={showFilter}
          onValueChange={setShowFilter}
        >
          Filter
        </Checkbox>
      </CardHeader>
      <CardBody className="gap-4">
        {showFilter ? (
          <DatePicker
            color="secondary"
            value={filterDate}
            onChange={setFilterDate as any}
          />
        ) : null}

        <Table
          isStriped
          removeWrapper
          aria-label="Example table with dynamic content"
        >
          <TableHeader columns={columns}>
            {(column) => (
              <TableColumn key={column.key}>{column.label}</TableColumn>
            )}
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
      </CardBody>
    </Card>
  );
}
function TransactionsTable(props: { selectedAccounts: string }) {
  const [betweenDates, setBetweenDates] = useState({
    start: now("utc").subtract({ days: 1 }),
    end: now("utc").add({ days: 1 }),
  });
  const { date_newest, date_oldest } = useMemo(
    () => ({
      date_newest: betweenDates.end.toDate().valueOf(),
      date_oldest: betweenDates.start.toDate().valueOf(),
    }),
    [betweenDates],
  );
  const { data: items, isLoading } = useAccountTransactions(
    props.selectedAccounts,
    date_newest,
    date_oldest,
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
      isStriped
      aria-label="Example table with dynamic content"
      bottomContent={
        <div className="flex flex-col gap-2 items-center">
          <div className={"".concat(pages > 1 ? "" : " hidden")}>
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
          <DateRangePicker
            className="max-w-md"
            label="Transactions between"
            value={betweenDates}
            onChange={setBetweenDates as any}
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
                <dd title={"unix milli: " + item.fullDate.toString()}>
                  {dayjs(item.fullDate).toISOString()}
                </dd>
                <dt>Custom timestamp:</dt>
                <dd title={"unix milli: " + item.fullDate2.toString()}>
                  {dayjs(item.fullDate2).toISOString()}
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
