import 'package:flutter/material.dart';
import 'package:flutter_dotenv/flutter_dotenv.dart';
import 'package:provider/provider.dart';

import 'models/comment.dart';
import 'services/api_service.dart';
import 'screens/home_screen.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await dotenv.load(fileName: ".env");
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) => CommentProvider(),
      child: MaterialApp(
        title: 'YouTube Commenter',
        theme: ThemeData(
          primarySwatch: Colors.red,
          visualDensity: VisualDensity.adaptivePlatformDensity,
        ),
        home: const HomeScreen(),
      ),
    );
  }
}

class CommentProvider extends ChangeNotifier {
  final ApiService _apiService = ApiService();
  List<Comment> _comments = [];
  bool _isLoading = false;
  String? _error;

  List<Comment> get comments => _comments;
  bool get isLoading => _isLoading;
  String? get error => _error;

  Future<void> fetchComments(String videoId) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _comments = await _apiService.getComments(videoId);
      _isLoading = false;
      notifyListeners();
    } catch (e) {
      _isLoading = false;
      _error = e.toString();
      notifyListeners();
    }
  }
}
