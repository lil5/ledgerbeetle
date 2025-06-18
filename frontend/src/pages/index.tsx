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
import Spreadsheet, {
  CellBase,
  createFormulaParser,
  Matrix,
} from "react-spreadsheet";
//@ts-ignore
import dayjs from "dayjs";
import { useDebounceValue } from "usehooks-ts";
import { InfoIcon } from "lucide-react";

//@ts-ignore
import { customFormulaParserFunctions } from "@/spreadsheetFormula";
import { useAccountNames } from "@/api/accountnames";
import DefaultLayout from "@/layouts/default";
import { useAccountTransactions } from "@/api/accounttransactions";
import { useAccountBalances } from "@/api/accountbalances";
import useStoreHook from "@/stores/hook";
import { selectedAccountsStore } from "@/stores/selected-accounts-store";
import Numberify from "@/components/numberify";

const customCreateFormulaParser = (data: Matrix<CellBase>) =>
  createFormulaParser(data, {
    functions: customFormulaParserFunctions,
  });

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

  const columnLabels = [
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
  ];
  const [rowLabels, setRowLabels] = useState<string[]>(() =>
    [...Array(30).keys()].map((v) => String(v + 1)),
  );
  const [data, setData] = useState<Matrix<CellBase<any>>>([
    [{ value: "Vanilla" }, { value: "Chocolate" }],
    [{ value: "Strawberry" }, { value: "Cookies" }],
  ]);

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
    }
    if (items.length) {
      for (let i = 0; i < units.length; i++) {
        const unit = units[i];
        const itemsMaxIndex = items.length + 1;

        arr.push([
          { value: i == 0 ? "Totals" : "" },
          { value: "" },
          { value: "" },
          { value: "" },
          {
            value: `=SUMIF(G2:G${itemsMaxIndex}, "${unit}", E2:E${itemsMaxIndex})`,
          },
          {
            value: `=SUMIF(G2:G${itemsMaxIndex}, "${unit}", F2:F${itemsMaxIndex})`,
          },
          { value: unit },
          { value: "" },
          { value: "" },
          { value: "" },
        ]);
      }
    }
    while (arr.length < items.length + 30) {
      arr.push([]);
    }
    setData(arr);
    setRowLabels([...Array(arr.length).keys()].map((v) => String(v + 1)));
  }, [items]);
  if (isLoading) return <Spinner />;

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-col gap-2 items-center">
        <DateRangePicker
          className="max-w-md"
          label="Transactions between"
          value={betweenDates}
          onChange={setBetweenDates as any}
        />
      </div>

      <div className="overflow-scroll w-[calc(100vw-1rem)] max-h-[80vh]">
        <Spreadsheet
          className="text-sm"
          columnLabels={columnLabels}
          createFormulaParser={customCreateFormulaParser}
          data={data}
          rowLabels={rowLabels}
          //@ts-expect-error types are incorrect
          setData={setData}
        />
      </div>
    </div>
  );
}
