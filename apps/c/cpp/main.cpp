#include <algorithm>
#include <iostream>
#include <list>
#include <map>
#include <random>
#include <vector>

extern "C" {
__attribute__((weak)) void *__dso_handle;
}

int main()
{

    // cpp version
    std::cout << "C++ version: " << __cplusplus << std::endl;

    // Create a list of integers
    std::list<int> myList = {5, 2, 8, 3, 1};

    // Print the original list
    std::cout << "Original list: ";
    for (const auto &num : myList) {
        std::cout << num << " ";
    }
    std::cout << std::endl;

    // Sort the list in ascending order
    myList.sort();

    // Print the sorted list
    std::cout << "Sorted list: ";
    for (const auto &num : myList) {
        std::cout << num << " ";
    }
    std::cout << std::endl;

    // Create a vector of integers
    std::vector<int> myVector = {5, 2, 8, 3, 1};

    // Print the original vector
    std::cout << "Original vector: ";
    for (const auto &num : myVector) {
        std::cout << num << " ";
    }
    std::cout << std::endl;

    // Sort the vector in ascending order
    std::sort(myVector.begin(), myVector.end());

    // Print the sorted vector
    std::cout << "Sorted vector: ";
    for (const auto &num : myVector) {
        std::cout << num << " ";
    }
    std::cout << std::endl;

    // Create a map of strings to integers
    std::map<std::string, int> myMap = {
        {"apple", 5}, {"banana", 2}, {"orange", 8}, {"grape", 3}, {"kiwi", 1}};

    // Print the map
    std::cout << "Map: ";
    for (const auto &pair : myMap) {
        std::cout << pair.first << ":" << pair.second << " ";
    }
    std::cout << std::endl;

    // random test
    std::random_device rd;
    std::mt19937 gen(rd());
    std::uniform_int_distribution<> dis(1, 100);
    for (int i = 0; i < 10; i++) {
        std::cout << "Random number: " << dis(gen) << std::endl;
    }
    return 0;
}
