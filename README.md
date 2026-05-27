# Trinity Language

Быстрый, современный язык программирования, объединяющий возможности C, C# и C++.

---

## Установка

### Быстрая (одна команда)

```powershell
irm https://raw.githubusercontent.com/ivan2002312/Trinity-Language/refs/heads/main/install.ps1 | iex
```
Проверка
```powershell
trinity --help
```
Первая программа
```Создайте файл hello.tr:
module Hello;

static int main() {
    println("Hello, World!");
    return 0;
}
```

```Запустите:

powershell
trinity hello.tr --run
```
Основы языка
Комментарии

// Однострочный комментарий

/*
   Многострочный
   комментарий
*/
Переменные
```trinity
// Автовывод типа
var x = 42;          // int
var name = "Trinity"; // string
var flag = true;      // bool
var pi = 3.14;        // float

// Явный тип
int age = 25;
string city = "Moscow";
bool active = false;
Типы данных
Тип	Описание	Пример
int	Целое 32 бита	42
i64	Целое 64 бита	9999999999
float	Дробное 32 бита	3.14
f64	Дробное 64 бита	3.1415926535
bool	Логический	true, false
char	Символ	'A'
string	Строка	"Hello"
void	Пустой тип	—
Операторы
trinity
// Арифметика
var sum = a + b;
var diff = a - b;
var prod = a * b;
var quot = a / b;
var rem = a % b;

// Сравнение
var eq = a == b;
var neq = a != b;
var lt = a < b;
var gt = a > b;
var le = a <= b;
var ge = a >= b;

// Логические
var and = a && b;
var or = a || b;
var not = !a;

// Присваивание с операцией
x += 5;  // x = x + 5
x -= 3;  // x = x - 3
x *= 2;  // x = x * 2
x /= 4;  // x = x / 4

// Инкремент/декремент
x++;
x--;
Управление потоком
if / else
trinity
if (x > 10) {
    println("Больше 10");
} else if (x > 5) {
    println("Больше 5");
} else {
    println("5 или меньше");
}
Тернарный оператор
trinity
var max = a > b ? a : b;
while
trinity
var i = 0;
while (i < 5) {
    println(i);
    i = i + 1;
}
for
trinity
// Классический for
for (var i = 0; i < 10; i = i + 1) {
    println(i);
}

// Бесконечный цикл с break
for (;;) {
    if (условие) break;
}
foreach
trinity
var numbers = [1, 2, 3, 4, 5];
foreach (var n in numbers) {
    println(n);
}
switch
trinity
switch (value) {
    case 1: 
        println("Один"); 
        break;
    case 2: 
        println("Два"); 
        break;
    case 3:
    case 4:
        println("Три или четыре");
        break;
    default: 
        println("Другое");
}
Функции
Объявление
trinity
// Простая функция
static int add(int a, int b) {
    return a + b;
}

// Без возврата
static void say_hello(string name) {
    println("Hello, " + name + "!");
}

// Без параметров
static int get_answer() {
    return 42;
}

// С параметрами по умолчанию
static void greet(string name = "Гость") {
    println("Привет, " + name);
}
Вызов
trinity
var result = add(10, 20);
say_hello("Мир");
var answer = get_answer();
greet();         // "Привет, Гость"
greet("Иван");   // "Привет, Иван"
Рекурсия
trinity
static int factorial(int n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
Массивы
Создание
trinity
// Пустой массив заданного размера
var arr = new int[5];

// С литералами
var numbers = [1, 2, 3, 4, 5];
var names = ["Анна", "Борис", "Виктор"];
var mixed = [1, "два", true];

// Многомерный
var matrix = new int[3, 3];
Доступ
trinity
arr[0] = 10;
var first = arr[0];
var last = arr[arr.len() - 1];
Свойства
trinity
var length = arr.len();  // длина массива
Строки
trinity
// Создание
var s = "Hello";
var empty = "";

// Конкатенация
var greeting = "Hello, " + name + "!";

// Сравнение
if (s == "Hello") { ... }

// Длина
var len = s.len();

// Доступ к символам
var first = s[0];
var last = s[s.len() - 1];
Классы
Объявление
trinity
class Person {
    // Поля
    string name;
    int age;
    
    // Конструктор
    Person(string name, int age) {
        this.name = name;
        this.age = age;
    }
    
    // Метод
    void introduce() {
        println("Я " + name + ", мне " + age + " лет");
    }
    
    // Метод с возвратом
    int get_age() {
        return age;
    }
}
```
Использование
```trinity
var person = new Person("Иван", 25);
person.introduce();
var age = person.get_age();
Обработка ошибок
try / catch / finally
trinity
try {
    // Опасный код
    var result = 10 / 0;
} catch (var error) {
    println("Ошибка: " + error);
} finally {
    println("Выполнится всегда");
}
throw
```
```trinity
static void check_age(int age) {
    if (age < 0) {
        throw "Возраст не может быть отрицательным";
    }
    if (age > 150) {
        throw "Нереальный возраст";
    }
}
Встроенные функции
Ввод/вывод
```
```trinity
print("Без перевода строки");
println("С переводом строки");
println("Значение: ", переменная);

var input = read_line();  // Чтение с клавиатуры
Системные
```
```trinity
var size = sizeof(int);     // Размер типа в байтах
var type = typeof(x);       // Тип переменной
var name = nameof(x);       // Имя переменной как строка
Шаблоны (Generics)
```
```trinity
// Шаблонная функция
template <T>
static T max(T a, T b) {
    if (a > b) return a;
    return b;
}

// Использование
var max_int = max(10, 20);
var max_float = max(3.14, 2.71);
```
Примеры программ
Калькулятор
```trinity
module Calculator;

static int main() {
    var a = 10;
    var b = 3;
    
    println("a = ", a);
    println("b = ", b);
    println("a + b = ", a + b);
    println("a - b = ", a - b);
    println("a * b = ", a * b);
    println("a / b = ", a / b);
    println("a % b = ", a % b);
    
    return 0;
}
```
Таблица умножения
```trinity
module Multiplication;

static int main() {
    for (var i = 1; i <= 9; i = i + 1) {
        for (var j = 1; j <= 9; j = j + 1) {
            print(i * j, "\t");
        }
        println("");
    }
    return 0;
}
```
Числа Фибоначчи
```trinity
module Fibonacci;

static int fib(int n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

static int main() {
    println("Первые 10 чисел Фибоначчи:");
    for (var i = 0; i < 10; i = i + 1) {
        println("F(", i, ") = ", fib(i));
    }
    return 0;
}
FizzBuzz
```
```trinity
module FizzBuzz;

static int main() {
    for (var i = 1; i <= 30; i = i + 1) {
        if (i % 15 == 0) {
            println("FizzBuzz");
        } else if (i % 3 == 0) {
            println("Fizz");
        } else if (i % 5 == 0) {
            println("Buzz");
        } else {
            println(i);
        }
    }
    return 0;
}
```
Угадай число
```trinity
module GuessNumber;

static int main() {
    var secret = 42;
    var guess = 0;
    var attempts = 0;
    
    println("Угадайте число от 1 до 100!");
    
    while (guess != secret) {
        print("Ваш вариант: ");
        guess = read_line();
        attempts = attempts + 1;
        
        if (guess < secret) {
            println("Больше!");
        } else if (guess > secret) {
            println("Меньше!");
        }
    }
    
    println("Правильно! Попыток: ", attempts);
    return 0;
}
```
```
Команды
bash
trinity file.tr --run         # Запуск программы
trinity file.tr --lex-only    # Показать токены
trinity file.tr --parse-only  # Показать AST
Ошибки и их решение
Ошибка	Причина	Решение
No main	Нет функции main	Добавьте static int main()
Undefined: x	Переменная не объявлена	Объявите через var
Cannot add	Разные типы	Приведите типы явно
Index OOB	Выход за границы массива	Проверьте индекс
```
Часто задаваемые вопросы
В: Нужна ли точка с запятой?
О: Да, как в C/C++/C#.

В: Есть ли сборщик мусора?
О: Пока ручное управление, GC планируется.

В: Можно ли писать без классов?
О: Да, функции могут быть на верхнем уровне.

В: Как подключить внешний код?
О: Используйте import Module.Name;

Модуль — это файл с расширением `.trm`. Назовите его, например, `mymodule.trm`:

```trinity
// mymodule.trm
module mymodule;

class MyFuncs {
    static int double(int x) {
        return x * 2;
    }
    
    static string greet(string name) {
        return "Hello, " + name + "!";
    }
    
    static int factorial(int n) {
        if (n <= 1) return 1;
        return n * factorial(n - 1);
    }
}
```
Правила:

Первая строка: module имя_модуля;

Все функции должны быть внутри class

Все методы должны быть static

Имя класса может быть любым

Шаг 2: Поместите модуль в проект
Положите файл .trm рядом с вашей программой:

```text
myproject/
├── main.tr
└── mymodule.trm
```
Шаг 3: Импортируйте и используйте
```trinity
// main.tr
module MyProject;
import mymodule;

class App {
    static int main() {
        println(mymodule.greet("World"));     // Hello, World!
        println(mymodule.double(21));         // 42
        println(mymodule.factorial(5));       // 120
        return 0;
    }
}
```
Вызов: имя_модуля.имя_функции(аргументы)

Шаг 4: Запустите
```powershell
trinity main.tr --run
```
Где Trinity ищет модули
Рядом с программой — ./mymodule.trm

В папке packages — ./packages/mymodule/src/mymodule.trm

Создание пакета для распространения
Структура пакета:

```text
mymodule/
├── trinity.json        # Метаданные
└── src/
    └── mymodule.trm    # Код модуля
```
trinity.json:

```json
{
    "name": "mymodule",
    "version": "0.1.0",
    "description": "My awesome module",
    "author": "Your Name"
}
```
```text
Частые ошибки
Ошибка	Решение
module не найдено	Проверьте имя файла: math.trm, а не math.tr
Функция не найдена	Функции должны быть static и внутри class
Unknown: math.add	Импорт работает? Проверьте import math;
```
# Trinity Package Manager (trpip)

Пакетный менеджер для языка Trinity. Устанавливает модули из GitHub репозитория [Trinity-Module](https://github.com/ivan2002312/Trinity-Module).

---

## Команды

### Поиск модулей

```powershell
trinity trpip search
trinity trpip search math
Ищет модули в репозитории Trinity-Module.
```

Установка модуля
```powershell
trinity trpip install math
trinity trpip install strings
trinity trpip install openai
Скачивает .trm файл из репозитория в папку modules/ рядом с trinity.exe.
```

Список установленных
```powershell
trinity trpip list
Показывает все установленные модули и путь к ним.
```

Удаление модуля
```powershell
trinity trpip remove math
Удаляет модуль из папки modules/.
```

Обновление модуля
```powershell
trinity trpip install math
```
Просто установите заново — модуль перезапишется.

Где хранятся модули
Модули скачиваются в папку modules/ рядом с trinity.exe:

```text
C:\Users\Имя\.cargo\bin\
├── trinity.exe
└── modules/
    ├── math/
    │   └── math.trm
    ├── strings/
    │   └── strings.trm
    └── openai/
        └── openai.trm
```
Как Trinity находит модули
При import math; Trinity ищет модуль в трёх местах:

Рядом с программой — ./math.trm

В папке src — ./src/math.trm

В папке modules — ./modules/math/math.trm

Если модуль не найден локально, Trinity пытается автоматически скачать его из репозитория!

Использование модуля
```trinity
module MyApp;
import math;  // Trinity сама скачает math, если его нет

class App {
    static int main() {
        println("2 + 2 = ", math.add(2, 2));
        println("5 * 3 = ", math.mul(5, 3));
        return 0;
    }
}
Создание своего модуля
Создайте файл .trm:
```

```trinity
// mymodule.trm
module mymodule;

class MyFuncs {
    static int double(int x) {
        return x * 2;
    }
    
    static string greet(string name) {
        return "Hello, " + name + "!";
    }
}
```
Положите рядом с программой или установите через trpip.

Чтобы опубликовать модуль — сделайте Pull Request в Trinity-Module.

# Компиляция Trinity программ в EXE

Trinity поддерживает компиляцию программ в standalone `.exe` файлы.

---

## Простая компиляция

```powershell
trinity myapp.tr --build
Создаст myapp.exe — независимый исполняемый файл.
```

Структура программы для компиляции
```trinity
module MyApp;

class App {
    static int main() {
        println("Hello from EXE!");
        
        var x = 42;
        var y = 10;
        var sum = x + y;
        println("Sum = ", sum);
        
        return 0;
    }
}
```

Поддерживается в компиляции
Возможность	Статус
Переменные (var)	✅
Арифметика (+, -, *, /)	✅
println / print	✅
if / else	✅
while	✅
for	✅
return	✅
Строки	✅
Числа	✅
Модули (import)	✅
read_line()	✅
Ограничения компиляции
Параметры функций — только int

Модули компилируются как заглушки

Нет поддержки float, массивов в сгенерированном коде

Сложные функции модулей требуют интерпретатора (--run)

Компиляция с модулями
```trinity
module MyApp;
import math;  // Модуль math.trm

class App {
    static int main() {
        var result = math.add(10, 20);
        println(result);
        return result;
    }
}
```
```powershell
trinity myapp.tr --build
.\myapp.exe
```
Модуль автоматически загрузится и скомпилируется.

Интерпретатор vs Компилятор
--run	--build
Скорость запуска	Мгновенно	Требует сборки
Все возможности	✅	Базовые
Модули	Полная поддержка	Заглушки
Размер файла	—	~500 KB
Отладка	Сообщения об ошибках	Ошибки Rust
Рекомендации
Разработка: trinity file.tr --run

Релиз: trinity file.tr --build

Модули: Используйте --run

Простой код: Используйте --build
