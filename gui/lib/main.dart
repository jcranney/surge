import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

String url = "http://surge.local:5000";

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.deepPurple,
          brightness: Brightness.dark,
        ),
        brightness: Brightness.dark,
      ),
      home: const MyHomePage(title: 'Telescope Mount Control'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});
  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  bool debugMode = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text("/dev/null mode"),
                SizedBox(width: 10),
                Checkbox(
                  value: debugMode,
                  onChanged: (value) => setState(() {
                    if (value != null) {
                      debugMode = value;
                    }
                  }),
                ),
              ],
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                MotorColumn(motor: 'A', debugMode: debugMode),
                SizedBox(width: 20.0),
                MotorColumn(motor: 'B', debugMode: debugMode),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class DropRow {
  final List<String> items;
  final String item;
  final void Function(String?) onChanged;
  final String label;
  const DropRow({
    required this.items,
    required this.item,
    required this.onChanged,
    required this.label,
  });

  TableRow build(BuildContext context) {
    return TableRow(
      children: [
        Text("$label:", style: TextStyle(fontSize: 16)),
        DropdownButton(
          isExpanded: true,
          items: items
              .map(
                (value) => DropdownMenuItem(value: value, child: Text(value)),
              )
              .toList(),
          value: item,
          onChanged: onChanged,
        ),
      ],
    );
  }
}

class MotorColumn extends StatefulWidget {
  final String motor;
  final bool debugMode;
  const MotorColumn({super.key, required this.motor, required this.debugMode});

  @override
  State<MotorColumn> createState() => _MotorColumnState();
}

class _MotorColumnState extends State<MotorColumn> {
  final directions = ["forward", "backward"];
  late String direction;
  final buffers = ["period", "hightime"];
  late String buffer;
  final TextEditingController valueController = TextEditingController();
  Future<http.Response>? response;

  void stopMotor() async {
    var data = {"motor": widget.motor.toLowerCase()};
    if (widget.debugMode) {
      data["device"] = "/dev/null";
    }
    setState(() {
      response = http.post(Uri.http(url, "/stop", data));
    });
  }

  void commandMotor() async {
    var data = {
      "motor": widget.motor.toLowerCase(),
      "direction": direction,
      "enabled": "true",
      "buffer": buffer,
      "value": valueController.value.text,
    };
    if (widget.debugMode) {
      data["device"] = "/dev/null";
    }
    setState(() {
      response = http.post(Uri.http(url, "/command", data));
    });
  }

  @override
  void initState() {
    super.initState();
    direction = directions[0];
    buffer = buffers[0];
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: response,
      builder: (context, snapshot) {
        final List<Widget> column = [
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Text(
              "Motor ${widget.motor}",
              style: TextStyle(fontSize: 20),
            ),
          ),
          SizedBox(
            width: 220.0,
            child: Table(
              defaultVerticalAlignment: TableCellVerticalAlignment.middle,
              children: <TableRow>[
                DropRow(
                  label: "Direction",
                  items: directions,
                  item: direction,
                  onChanged: (value) => setState(() {
                    if (value != null) {
                      direction = value;
                    }
                  }),
                ).build(context),
                DropRow(
                  label: "Buffer",
                  items: buffers,
                  item: buffer,
                  onChanged: (value) => setState(() {
                    if (value != null) {
                      buffer = value;
                    }
                  }),
                ).build(context),
                TableRow(
                  children: [
                    Text("Buffer Value:", style: TextStyle(fontSize: 16)),
                    TextField(
                      keyboardType: TextInputType.numberWithOptions(),
                      controller: valueController,
                      decoration: InputDecoration(
                        hint: Text(
                          "1000",
                          style: TextStyle(
                            color: const Color.fromARGB(255, 182, 180, 180),
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),

          Padding(
            padding: const EdgeInsets.all(16.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                FloatingActionButton.extended(
                  onPressed: commandMotor,
                  icon: Icon(Icons.send),
                  label: Text("command"),
                ),
                SizedBox(width: 10.0),
                FloatingActionButton.extended(
                  onPressed: stopMotor,
                  icon: Icon(Icons.cancel),
                  label: Text("STOP"),
                ),
              ],
            ),
          ),
        ];
        if (snapshot.hasData) {
          column.add(Text("response: ${snapshot.data!.statusCode}"));
          if (snapshot.data!.statusCode != 200) {
            print(snapshot.data!.body);
          }
        } else {
          column.add(Text(" "));
        }
        return Column(
          mainAxisAlignment: MainAxisAlignment.start,
          children: column,
        );
      },
    );
  }
}
