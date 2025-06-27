import { useEffect, useMemo, useState } from "react";
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
import { CellBase, Matrix, Spreadsheet } from "@lil5/react-spreadsheet";
//@ts-ignore
import dayjs from "dayjs";
import * as XLSX from "xlsx";
import { useDebounceValue } from "usehooks-ts";
import { InfoIcon, LucideDownload } from "lucide-react";
import { useStore } from "@nanostores/react";

import { useAccountNames } from "@/api/accountnames";
import DefaultLayout from "@/layouts/default";
import { useAccountTransactions } from "@/api/accounttransactions";
import { useAccountBalances } from "@/api/accountbalances";
import { $selectedAccountsStore } from "@/stores/selected-accounts-store";
import Numberify from "@/components/numberify";

export default function IndexPage() {
  const queryAccountNames = useAccountNames();
  const [isOpenTooltip, setIsOpenTooltip] = useState(false);
  const search = useStore($selectedAccountsStore);
  const setSearch = (s: typeof search) => $selectedAccountsStore.set(s);
  const [searchDebounce] = useDebounceValue(search, 1300, { trailing: true });
  const [selectedAccountNames, setSelectedAccountNames] = useState(
    new Set([] as string[]),
  );
  const items = useMemo<Array<{ text: string }>>(() => {
    if (!queryAccountNames.data) return [];

    return queryAccountNames.data.map((text) => ({ text }));
  }, [queryAccountNames]);

  const _setSelectedAccountNames = (s: Set<string> | "all") => {
    if (s === "all") s = new Set(queryAccountNames.data || []);
    setSelectedAccountNames(s as Set<string>);

    setSearch([...s].map((s) => s + "**").join("|"));
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
                    <div className="px-1 py-2 space-y-2">
                      <div className="text-small font-bold">Filter symbols</div>
                      <ul className="space-y-2 text-tiny">
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
                      <div className="text-small font-bold">
                        Account prefix symbols
                      </div>
                      <ul className="space-y-2 text-tiny">
                        <li>
                          <Code className="text-xs">a</Code> Assets
                        </li>
                        <li>
                          <Code className="text-xs">l</Code> Liabilities
                        </li>
                        <li>
                          <Code className="text-xs">e</Code> Equity
                        </li>
                        <li>
                          <Code className="text-xs">r</Code> Revenues
                        </li>
                        <li>
                          <Code className="text-xs">x</Code> Expenses
                        </li>
                      </ul>
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
              onValueChange={setSearch}
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
            value={filterDate as any}
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

  const [data, setData] = useState<Matrix<CellBase<any>>>([]);

  useEffect(() => {
    let units: string[] = [];
    const arr: typeof data = [
      [
        { value: "TransferID", readOnly: true },
        { value: "Created", readOnly: true },
        { value: "DebitAccount", readOnly: true },
        { value: "CreditAccount", readOnly: true },
        { value: "DebitAmount", readOnly: true },
        { value: "CreditAmount", readOnly: true },
        { value: "Unit", readOnly: true },
        { value: "Code", readOnly: true },
        { value: "DataRelatedID", readOnly: true },
        { value: "DataDate", readOnly: true },
      ],
    ];
    const colSize = arr[0].length;

    for (const item of items) {
      if (!units.includes(item.commodityUnit)) {
        units.push(item.commodityUnit);
      }
      arr.push([
        { value: item.transferId, readOnly: true },
        { value: dayjs(item.fullDate).toISOString(), readOnly: true },
        { value: item.debitAccount, readOnly: true },
        { value: item.creditAccount, readOnly: true },
        { value: item.debitAmount, readOnly: true },
        { value: item.creditAmount, readOnly: true },
        { value: item.commodityUnit, readOnly: true },
        { value: item.code, readOnly: true },
        { value: item.relatedId, readOnly: true },
        { value: dayjs(item.fullDate2).toISOString(), readOnly: true },
      ]);
      if (arr[arr.length - 1].length != colSize) {
        console.error(
          "Added invalid amount of columns",
          arr[arr.length - 1].length,
          colSize,
        );
      }
    }
    if (items.length) {
      for (let i = 0; i < units.length; i++) {
        const isFirst = i === 0;
        const unit = units[i];
        const itemsMaxIndex = items.length + 1;

        // prettier-ignore
        // eslint-disable-next-line prettier/prettier
        arr.push([
          { value: "" },
          { value: "" },
          { value: "" },
          { value: isFirst ? "Totals:" : "" },
          { value: `=SUMIF(G2:G${itemsMaxIndex}, "${unit}", E2:E${itemsMaxIndex})` },
          { value: `=SUMIF(G2:G${itemsMaxIndex}, "${unit}", F2:F${itemsMaxIndex})` },
          { value: unit },
          { value: "" },
          { value: "" },
          { value: "" },
        ]);
        if (arr[arr.length - 1].length != colSize) {
          console.error(
            "Added invalid amount of columns",
            arr[arr.length - 1].length,
            colSize,
          );
        }
      }
      arr.push(
        [
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
        ],
        [
          { value: "Transactions between:" },
          { value: betweenDates.start.toDate().toISOString() },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
        ],
        [
          { value: "" },
          { value: betweenDates.end.toDate().toISOString() },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
          { value: "" },
        ],
      );
    }
    for (let i = 0; i < 20; i++) {
      arr.push([
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
        { value: "" },
      ]);
      if (arr[arr.length - 1].length != colSize) {
        console.error(
          "Added invalid amount of columns",
          arr[arr.length - 1].length,
          colSize,
        );
      }
    }
    setData(arr);
  }, [items]);

  function onClickExport() {
    const sheet = XLSX.utils.aoa_to_sheet(
      data.map((row) =>
        row.map((cell) => {
          const v = cell!.value;

          if (String(v).startsWith("=")) {
            return { t: "n", f: v.slice(1) };
          }

          return cell!.value;
        }),
      ),
    );
    const wb = XLSX.utils.book_new();

    XLSX.utils.book_append_sheet(wb, sheet, "Sheet1", true);
    XLSX.writeFile(wb, "export.xlsx");
  }

  // const spreadsheetProps = useSpreadsheetTableProps(data, setData);

  if (isLoading) return <Spinner />;

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-col md:flex-row gap-2 justify-center items-center">
        <DateRangePicker
          className="max-w-md"
          label="Transactions between"
          value={betweenDates as any}
          onChange={setBetweenDates as any}
        />
        <Button onPress={onClickExport}>
          <LucideDownload />
          Export
        </Button>
      </div>

      <div className="overflow-scroll w-[calc(100vw-1rem)] max-h-[80vh]">
        <Spreadsheet
          className="text-sm"
          data={data}
          //  onChange={setData}
        />
      </div>
    </div>
  );
}
