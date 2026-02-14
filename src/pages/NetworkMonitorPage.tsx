import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    Activity,
    RefreshCw,
    Wifi,
    WifiOff,
    ArrowUpCircle,
    ArrowDownCircle,
    Globe,
    Server,
    EyeOff,
    Eye,
    Settings2,
    TrendingUp,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from "@/components/ui/tooltip";
import {
    Popover,
    PopoverContent,
    PopoverTrigger,
} from "@/components/ui/popover";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
    Tooltip as RechartsTooltip,
    Area,
    AreaChart,
    BarChart,
    Bar,
    XAxis,
    YAxis,
    ResponsiveContainer,
} from "recharts";

interface NetworkProcess {
    pid: number;
    name: string;
    tcp_connections: number;
    udp_connections: number;
    bytes_sent: number;
    bytes_received: number;
}

interface NetworkInterface {
    name: string;
    description: string;
    bytes_sent: number;
    bytes_received: number;
    status: string;
}

interface NetworkHistoryPoint {
    time: string;
    sent: number;
    received: number;
}

function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function formatBytesShort(bytes: number): string {
    if (bytes === 0) return "0";
    const k = 1024;
    const sizes = ["B", "K", "M", "G"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(0)) + sizes[i];
}

// Default filters for common virtual adapters
const DEFAULT_HIDDEN_KEYWORDS = ["VMware", "VirtualBox", "Hyper-V", "vEthernet", "Loopback"];

export function NetworkMonitorPage() {
    const [processes, setProcesses] = useState<NetworkProcess[]>([]);
    const [interfaces, setInterfaces] = useState<NetworkInterface[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [autoRefresh, setAutoRefresh] = useState(true);
    const [sortBy, setSortBy] = useState<"connections" | "sent" | "received">("connections");
    const [hiddenKeywords, setHiddenKeywords] = useState<string[]>(() => {
        const saved = localStorage.getItem("network_hidden_keywords");
        return saved ? JSON.parse(saved) : [];
    });
    const [networkHistory, setNetworkHistory] = useState<NetworkHistoryPoint[]>([]);
    const [prevTotals, setPrevTotals] = useState<{ sent: number; received: number } | null>(null);

    const loadData = useCallback(async () => {
        setIsLoading(true);
        try {
            const [procs, ifaces] = await Promise.all([
                invoke<NetworkProcess[]>("get_network_processes"),
                invoke<NetworkInterface[]>("get_network_interfaces"),
            ]);
            setProcesses(procs || []);
            setInterfaces(ifaces || []);

            // Calculate network speed
            const currentSent = (ifaces || []).reduce((sum, i) => sum + i.bytes_sent, 0);
            const currentReceived = (ifaces || []).reduce((sum, i) => sum + i.bytes_received, 0);

            if (prevTotals) {
                const sentSpeed = Math.max(0, (currentSent - prevTotals.sent) / 3); // bytes per second
                const receivedSpeed = Math.max(0, (currentReceived - prevTotals.received) / 3);

                setNetworkHistory(prev => {
                    const now = new Date();
                    const timeStr = `${now.getMinutes()}:${String(now.getSeconds()).padStart(2, '0')}`;
                    const newHistory = [...prev, { time: timeStr, sent: sentSpeed, received: receivedSpeed }];
                    // Keep last 20 points
                    return newHistory.slice(-20);
                });
            }
            setPrevTotals({ sent: currentSent, received: currentReceived });
        } catch (error) {
            console.error("Failed to load network data:", error);
        } finally {
            setIsLoading(false);
        }
    }, [prevTotals]);

    useEffect(() => {
        loadData();
    }, []);

    useEffect(() => {
        if (!autoRefresh) return;
        const interval = setInterval(loadData, 3000);
        return () => clearInterval(interval);
    }, [autoRefresh, loadData]);

    // Save hidden keywords to localStorage
    useEffect(() => {
        localStorage.setItem("network_hidden_keywords", JSON.stringify(hiddenKeywords));
    }, [hiddenKeywords]);

    const toggleKeyword = (keyword: string) => {
        setHiddenKeywords(prev =>
            prev.includes(keyword)
                ? prev.filter(k => k !== keyword)
                : [...prev, keyword]
        );
    };

    // Filter interfaces based on hidden keywords
    const filteredInterfaces = interfaces.filter(iface =>
        !hiddenKeywords.some(keyword =>
            iface.name.toLowerCase().includes(keyword.toLowerCase()) ||
            iface.description.toLowerCase().includes(keyword.toLowerCase())
        )
    );

    const sortedProcesses = [...processes].sort((a, b) => {
        switch (sortBy) {
            case "connections":
                return (b.tcp_connections + b.udp_connections) - (a.tcp_connections + a.udp_connections);
            case "sent":
                return b.bytes_sent - a.bytes_sent;
            case "received":
                return b.bytes_received - a.bytes_received;
            default:
                return 0;
        }
    });

    const totalConnections = processes.reduce(
        (sum, p) => sum + p.tcp_connections + p.udp_connections,
        0
    );

    const totalSent = filteredInterfaces.reduce((sum, i) => sum + i.bytes_sent, 0);
    const totalReceived = filteredInterfaces.reduce((sum, i) => sum + i.bytes_received, 0);

    // Get current speed from last history point
    const currentSentSpeed = networkHistory.length > 0 ? networkHistory[networkHistory.length - 1].sent : 0;
    const currentReceivedSpeed = networkHistory.length > 0 ? networkHistory[networkHistory.length - 1].received : 0;

    return (
        <div className="space-y-4 pb-12">
            {/* Header */}
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <Activity className="h-5 w-5 text-primary" />
                    <h2 className="text-lg font-semibold">Network Monitor</h2>
                </div>
                <div className="flex gap-2">
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <Button
                                variant={autoRefresh ? "default" : "outline"}
                                size="sm"
                                onClick={() => setAutoRefresh(!autoRefresh)}
                            >
                                {autoRefresh ? (
                                    <Wifi className="mr-1.5 h-3.5 w-3.5" />
                                ) : (
                                    <WifiOff className="mr-1.5 h-3.5 w-3.5" />
                                )}
                                {autoRefresh ? "Live" : "Paused"}
                            </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                            {autoRefresh ? "Auto-refreshing every 3s" : "Click to enable auto-refresh"}
                        </TooltipContent>
                    </Tooltip>
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={loadData}
                        disabled={isLoading}
                    >
                        <RefreshCw
                            className={`mr-1.5 h-3.5 w-3.5 ${isLoading ? "animate-spin" : ""}`}
                        />
                        Refresh
                    </Button>
                </div>
            </div>

            {/* Network Speed Graph */}
            <Card>
                <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="text-sm flex items-center gap-2">
                                <TrendingUp className="h-4 w-4 text-primary" />
                                Network Activity
                            </CardTitle>
                            <CardDescription className="text-xs">
                                Real-time network speed (bytes/sec)
                            </CardDescription>
                        </div>
                        <div className="flex gap-4 text-xs">
                            <div className="flex items-center gap-1.5">
                                <div className="h-2 w-2 rounded-full bg-purple-500" />
                                <span className="text-muted-foreground">Upload:</span>
                                <span className="font-mono text-purple-500">{formatBytes(currentSentSpeed)}/s</span>
                            </div>
                            <div className="flex items-center gap-1.5">
                                <div className="h-2 w-2 rounded-full bg-green-500" />
                                <span className="text-muted-foreground">Download:</span>
                                <span className="font-mono text-green-500">{formatBytes(currentReceivedSpeed)}/s</span>
                            </div>
                        </div>
                    </div>
                </CardHeader>
                <CardContent>
                    <div className="h-32 w-full">
                        {networkHistory.length > 1 ? (
                            <ResponsiveContainer width="100%" height="100%">
                                <AreaChart data={networkHistory}>
                                    <defs>
                                        <linearGradient id="sentGradient" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="#a855f7" stopOpacity={0.3} />
                                            <stop offset="95%" stopColor="#a855f7" stopOpacity={0} />
                                        </linearGradient>
                                        <linearGradient id="receivedGradient" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="#22c55e" stopOpacity={0.3} />
                                            <stop offset="95%" stopColor="#22c55e" stopOpacity={0} />
                                        </linearGradient>
                                    </defs>
                                    <XAxis dataKey="time" tick={{ fontSize: 10 }} stroke="#888" />
                                    <YAxis tick={{ fontSize: 10 }} stroke="#888" tickFormatter={formatBytesShort} />
                                    <RechartsTooltip
                                        contentStyle={{
                                            backgroundColor: 'hsl(var(--card))',
                                            border: '1px solid hsl(var(--border))',
                                            borderRadius: '8px',
                                            fontSize: '11px'
                                        }}
                                        labelStyle={{ color: 'hsl(var(--foreground))' }}
                                        formatter={(value: number) => formatBytes(value) + '/s'}
                                    />
                                    <Area
                                        type="monotone"
                                        dataKey="sent"
                                        stroke="#a855f7"
                                        strokeWidth={2}
                                        fill="url(#sentGradient)"
                                        name="Upload"
                                    />
                                    <Area
                                        type="monotone"
                                        dataKey="received"
                                        stroke="#22c55e"
                                        strokeWidth={2}
                                        fill="url(#receivedGradient)"
                                        name="Download"
                                    />
                                </AreaChart>
                            </ResponsiveContainer>
                        ) : (
                            <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
                                Collecting data... Graph will appear shortly.
                            </div>
                        )}
                    </div>
                </CardContent>
            </Card>

            {/* Statistics Cards */}
            <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
                <Card className="bg-gradient-to-br from-blue-500/10 to-blue-600/5 border-blue-500/20">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-2">
                            <Globe className="h-4 w-4 text-blue-500" />
                            <span className="text-xs text-muted-foreground">Connections</span>
                        </div>
                        <p className="text-2xl font-bold text-blue-500">{totalConnections}</p>
                    </CardContent>
                </Card>
                <Card className="bg-gradient-to-br from-green-500/10 to-green-600/5 border-green-500/20">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-2">
                            <Server className="h-4 w-4 text-green-500" />
                            <span className="text-xs text-muted-foreground">Processes</span>
                        </div>
                        <p className="text-2xl font-bold text-green-500">{processes.length}</p>
                    </CardContent>
                </Card>
                <Card className="bg-gradient-to-br from-purple-500/10 to-purple-600/5 border-purple-500/20">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-2">
                            <ArrowUpCircle className="h-4 w-4 text-purple-500" />
                            <span className="text-xs text-muted-foreground">Total Sent</span>
                        </div>
                        <p className="text-2xl font-bold text-purple-500">{formatBytes(totalSent)}</p>
                    </CardContent>
                </Card>
                <Card className="bg-gradient-to-br from-orange-500/10 to-orange-600/5 border-orange-500/20">
                    <CardContent className="pt-4">
                        <div className="flex items-center gap-2">
                            <ArrowDownCircle className="h-4 w-4 text-orange-500" />
                            <span className="text-xs text-muted-foreground">Total Received</span>
                        </div>
                        <p className="text-2xl font-bold text-orange-500">{formatBytes(totalReceived)}</p>
                    </CardContent>
                </Card>
            </div>

            {/* Network Interfaces */}
            <Card>
                <CardHeader className="pb-3">
                    <div className="flex items-center justify-between">
                        <div>
                            <CardTitle className="text-sm">Network Interfaces</CardTitle>
                            <CardDescription className="text-xs">
                                Active network adapters ({filteredInterfaces.length} of {interfaces.length})
                            </CardDescription>
                        </div>
                        <Popover>
                            <PopoverTrigger asChild>
                                <Button variant="outline" size="sm" className="h-7 text-xs">
                                    <Settings2 className="mr-1.5 h-3 w-3" />
                                    Filter
                                    {hiddenKeywords.length > 0 && (
                                        <Badge variant="secondary" className="ml-1.5 text-[10px] h-4 px-1">
                                            {hiddenKeywords.length}
                                        </Badge>
                                    )}
                                </Button>
                            </PopoverTrigger>
                            <PopoverContent className="w-56" align="end">
                                <div className="space-y-3">
                                    <div className="text-xs font-medium">Hide interfaces containing:</div>
                                    <div className="space-y-2">
                                        {DEFAULT_HIDDEN_KEYWORDS.map((keyword) => (
                                            <div key={keyword} className="flex items-center space-x-2">
                                                <Checkbox
                                                    id={keyword}
                                                    checked={hiddenKeywords.includes(keyword)}
                                                    onCheckedChange={() => toggleKeyword(keyword)}
                                                />
                                                <Label htmlFor={keyword} className="text-xs cursor-pointer">
                                                    {keyword}
                                                </Label>
                                            </div>
                                        ))}
                                    </div>
                                    {hiddenKeywords.length > 0 && (
                                        <Button
                                            variant="ghost"
                                            size="sm"
                                            className="w-full h-7 text-xs"
                                            onClick={() => setHiddenKeywords([])}
                                        >
                                            <Eye className="mr-1.5 h-3 w-3" />
                                            Show All
                                        </Button>
                                    )}
                                </div>
                            </PopoverContent>
                        </Popover>
                    </div>
                </CardHeader>
                <CardContent>
                    {filteredInterfaces.length > 0 ? (
                        <div className="space-y-3">
                            {filteredInterfaces.map((iface, index) => (
                                <div
                                    key={index}
                                    className="rounded-lg border bg-muted/30 p-3"
                                >
                                    <div className="flex items-center justify-between mb-2">
                                        <div className="flex items-center gap-2">
                                            <Wifi className="h-4 w-4 text-green-500" />
                                            <span className="text-sm font-medium">{iface.name}</span>
                                            <Badge variant="outline" className="text-[10px]">
                                                {iface.status}
                                            </Badge>
                                        </div>
                                    </div>
                                    <p className="text-xs text-muted-foreground mb-2 truncate">
                                        {iface.description}
                                    </p>
                                    <div className="grid grid-cols-2 gap-4">
                                        <div>
                                            <div className="flex items-center justify-between text-xs mb-1">
                                                <span className="text-muted-foreground">Sent</span>
                                                <span className="text-purple-500 font-medium">
                                                    {formatBytes(iface.bytes_sent)}
                                                </span>
                                            </div>
                                            <Progress value={30} className="h-1" />
                                        </div>
                                        <div>
                                            <div className="flex items-center justify-between text-xs mb-1">
                                                <span className="text-muted-foreground">Received</span>
                                                <span className="text-orange-500 font-medium">
                                                    {formatBytes(iface.bytes_received)}
                                                </span>
                                            </div>
                                            <Progress value={60} className="h-1" />
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <div className="flex h-20 items-center justify-center text-xs text-muted-foreground">
                            {isLoading ? "Loading interfaces..." : interfaces.length > 0 ? (
                                <div className="flex items-center gap-2">
                                    <EyeOff className="h-4 w-4" />
                                    All interfaces are hidden. Adjust filters to show.
                                </div>
                            ) : "No active interfaces found"}
                        </div>
                    )}
                </CardContent>
            </Card>

            {/* Process Connections Graph */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <Card className="md:col-span-1">
                    <CardHeader className="pb-2">
                        <CardTitle className="text-sm">Top Processes by Connections</CardTitle>
                        <CardDescription className="text-xs">
                            Active TCP/UDP sockets
                        </CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="h-[250px] w-full">
                            {sortedProcesses.length > 0 ? (
                                <ResponsiveContainer width="100%" height="100%">
                                    <BarChart
                                        layout="vertical"
                                        data={sortedProcesses.slice(0, 10)}
                                        margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
                                    >
                                        <XAxis type="number" hide />
                                        <YAxis
                                            dataKey="name"
                                            type="category"
                                            width={100}
                                            tick={{ fontSize: 10 }}
                                            tickFormatter={(val) => val.length > 15 ? val.substring(0, 15) + '...' : val}
                                        />
                                        <RechartsTooltip
                                            contentStyle={{
                                                backgroundColor: 'hsl(var(--card))',
                                                border: '1px solid hsl(var(--border))',
                                                borderRadius: '8px',
                                                fontSize: '11px'
                                            }}
                                            labelStyle={{ color: 'hsl(var(--foreground))' }}
                                            cursor={{ fill: 'hsl(var(--muted)/0.4)' }}
                                            formatter={(value: number) => [value, 'Connections']}
                                        />
                                        <Bar dataKey="tcp_connections" stackId="a" fill="hsl(217, 91%, 60%)" radius={[0, 4, 4, 0]} name="TCP" />
                                        <Bar dataKey="udp_connections" stackId="a" fill="hsl(142, 71%, 45%)" radius={[0, 4, 4, 0]} name="UDP" />
                                    </BarChart>
                                </ResponsiveContainer>
                            ) : (
                                <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
                                    No active processes
                                </div>
                            )}
                        </div>
                    </CardContent>
                </Card>

                {/* Process List Table */}
                <Card className="md:col-span-2">
                    <CardHeader className="pb-3">
                        <div className="flex items-center justify-between">
                            <div>
                                <CardTitle className="text-sm">Network Activity by Process</CardTitle>
                                <CardDescription className="text-xs">
                                    Processes with active network connections
                                </CardDescription>
                            </div>
                            <div className="flex gap-1">
                                <Button
                                    variant={sortBy === "connections" ? "default" : "ghost"}
                                    size="sm"
                                    className="h-6 text-xs px-2"
                                    onClick={() => setSortBy("connections")}
                                >
                                    Connections
                                </Button>
                            </div>
                        </div>
                    </CardHeader>
                    <CardContent className="p-0">
                        {sortedProcesses.length > 0 ? (
                            <ScrollArea className="h-[280px]">
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead className="text-xs">Process</TableHead>
                                            <TableHead className="text-xs text-center">PID</TableHead>
                                            <TableHead className="text-xs text-center">TCP</TableHead>
                                            <TableHead className="text-xs text-center">UDP</TableHead>
                                            <TableHead className="text-xs text-right text-muted-foreground/50">Sent (N/A)</TableHead>
                                            <TableHead className="text-xs text-right text-muted-foreground/50">Recv (N/A)</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {sortedProcesses.map((proc, index) => (
                                            <TableRow key={index} className="hover:bg-muted/50">
                                                <TableCell className="text-xs font-medium py-2">
                                                    {proc.name}
                                                </TableCell>
                                                <TableCell className="text-xs text-center text-muted-foreground py-2">
                                                    {proc.pid}
                                                </TableCell>
                                                <TableCell className="text-xs text-center py-2">
                                                    <Badge variant="secondary" className="text-[10px] bg-blue-500/10 text-blue-600 hover:bg-blue-500/20">
                                                        {proc.tcp_connections}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell className="text-xs text-center py-2">
                                                    <Badge variant="outline" className="text-[10px] text-green-600 border-green-200">
                                                        {proc.udp_connections}
                                                    </Badge>
                                                </TableCell>
                                                <TableCell className="text-xs text-right text-muted-foreground/30 py-2 font-mono">
                                                    -
                                                </TableCell>
                                                <TableCell className="text-xs text-right text-muted-foreground/30 py-2 font-mono">
                                                    -
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </ScrollArea>
                        ) : (
                            <div className="flex h-32 items-center justify-center text-xs text-muted-foreground">
                                {isLoading ? "Loading processes..." : "No network activity detected"}
                            </div>
                        )}
                    </CardContent>
                </Card>
            </div>
        </div>
    );
}
