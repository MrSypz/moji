-- phpMyAdmin SQL Dump
-- version 5.2.1deb3
-- https://www.phpmyadmin.net/
--
-- Host: localhost:3306
-- Generation Time: Sep 07, 2025 at 11:41 PM
-- Server version: 8.0.43-0ubuntu0.24.04.1
-- PHP Version: 8.3.6

SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";


/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;

--
-- Database: `moji`
--

-- --------------------------------------------------------

--
-- Table structure for table `tb_config`
--

CREATE TABLE `tb_config` (
  `id` int NOT NULL,
  `config_key` varchar(100) COLLATE utf8mb4_unicode_ci NOT NULL,
  `config_value` decimal(10,4) NOT NULL,
  `description` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

--
-- Dumping data for table `tb_config`
--

INSERT INTO `tb_config` (`id`, `config_key`, `config_value`, `description`, `created_at`, `updated_at`) VALUES
(1, 'market_vat_rate', 0.3400, 'VAT rate for market transactions (34%)', '2025-08-25 09:33:39', '2025-08-25 09:33:39'),
(2, 'transfer_fee_rate', 0.1000, 'Default transfer fee rate (10%)', '2025-08-25 09:33:39', '2025-08-25 09:33:39'),
(3, 'wallet_to_bank_fee_rate', 0.0500, 'Wallet to bank transfer fee when amount >= 10000 (5%)', '2025-08-25 09:33:39', '2025-08-25 09:33:39'),
(4, 'wallet_to_bank_threshold', 10000.0000, 'Threshold amount for special wallet to bank fee', '2025-08-25 09:33:39', '2025-08-25 09:33:39'),
(5, 'market_transaction_fee', 0.0200, 'Market transaction fee (2%)', '2025-08-25 09:33:39', '2025-08-25 09:33:39');

-- --------------------------------------------------------

--
-- Table structure for table `tb_market_items`
--

CREATE TABLE `tb_market_items` (
  `id` int NOT NULL,
  `item_key` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `item_name` varchar(100) COLLATE utf8mb4_unicode_ci NOT NULL,
  `base_price` bigint NOT NULL,
  `current_sell_price` bigint NOT NULL,
  `current_buy_price` bigint NOT NULL,
  `total_sold` bigint NOT NULL DEFAULT '0',
  `total_bought` bigint NOT NULL DEFAULT '0',
  `price_multiplier` double NOT NULL DEFAULT '1',
  `last_price_update` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ;

--
-- Dumping data for table `tb_market_items`
--

INSERT INTO `tb_market_items` (`id`, `item_key`, `item_name`, `base_price`, `current_sell_price`, `current_buy_price`, `total_sold`, `total_bought`, `price_multiplier`, `last_price_update`, `created_at`, `updated_at`) VALUES
(1, 'minecraft:wheat', 'Wheat', 100, 92, 147, 6925, 0, 0.911370618803475, '2025-08-25 13:15:49', '2025-08-25 07:36:22', '2025-08-25 13:15:49'),
(2, 'minecraft:sugar_cane', 'Sugar Cane', 80, 78, 125, 383, 0, 0.9764908581423248, '2025-08-25 13:15:49', '2025-08-25 07:36:22', '2025-08-25 13:15:49'),
(3, 'minecraft:pumpkin', 'Pumpkin', 300, 299, 479, 5, 0, 0.997473304225, '2025-08-25 13:15:49', '2025-08-25 12:58:07', '2025-08-25 13:15:49');

-- --------------------------------------------------------

--
-- Table structure for table `tb_market_transactions`
--

CREATE TABLE `tb_market_transactions` (
  `id` bigint NOT NULL,
  `player_uuid` varchar(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `item_key` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `transaction_type` enum('BUY','SELL') COLLATE utf8mb4_unicode_ci NOT NULL,
  `quantity` int NOT NULL,
  `price_per_unit` bigint NOT NULL,
  `total_amount` bigint NOT NULL,
  `price_multiplier` double NOT NULL,
  `timestamp` timestamp NULL DEFAULT CURRENT_TIMESTAMP
) ;

-- --------------------------------------------------------

--
-- Table structure for table `tb_user`
--

CREATE TABLE `tb_user` (
  `id` int NOT NULL,
  `player_uuid` varchar(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `player_name` varchar(16) COLLATE utf8mb4_unicode_ci NOT NULL,
  `wallet` bigint NOT NULL DEFAULT '0',
  `bank` bigint NOT NULL DEFAULT '0',
  `is_bank_open` tinyint(1) NOT NULL DEFAULT '0',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ;

--
-- Indexes for dumped tables
--

--
-- Indexes for table `tb_config`
--
ALTER TABLE `tb_config`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `config_key` (`config_key`),
  ADD KEY `idx_config_key` (`config_key`);

--
-- Indexes for table `tb_market_items`
--
ALTER TABLE `tb_market_items`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `item_key` (`item_key`),
  ADD KEY `idx_item_key` (`item_key`);

--
-- Indexes for table `tb_market_transactions`
--
ALTER TABLE `tb_market_transactions`
  ADD PRIMARY KEY (`id`),
  ADD KEY `idx_player_uuid` (`player_uuid`),
  ADD KEY `idx_item_key` (`item_key`),
  ADD KEY `idx_transaction_type` (`transaction_type`),
  ADD KEY `idx_timestamp` (`timestamp`),
  ADD KEY `idx_item_type_time` (`item_key`,`transaction_type`,`timestamp`);

--
-- Indexes for table `tb_user`
--
ALTER TABLE `tb_user`
  ADD PRIMARY KEY (`id`),
  ADD UNIQUE KEY `player_uuid` (`player_uuid`),
  ADD KEY `idx_player_uuid` (`player_uuid`),
  ADD KEY `idx_player_name` (`player_name`);

--
-- AUTO_INCREMENT for dumped tables
--

--
-- AUTO_INCREMENT for table `tb_config`
--
ALTER TABLE `tb_config`
  MODIFY `id` int NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=6;

--
-- AUTO_INCREMENT for table `tb_market_items`
--
ALTER TABLE `tb_market_items`
  MODIFY `id` int NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `tb_market_transactions`
--
ALTER TABLE `tb_market_transactions`
  MODIFY `id` bigint NOT NULL AUTO_INCREMENT;

--
-- AUTO_INCREMENT for table `tb_user`
--
ALTER TABLE `tb_user`
  MODIFY `id` int NOT NULL AUTO_INCREMENT;

--
-- Constraints for dumped tables
--

--
-- Constraints for table `tb_market_transactions`
--
ALTER TABLE `tb_market_transactions`
  ADD CONSTRAINT `tb_market_transactions_ibfk_1` FOREIGN KEY (`item_key`) REFERENCES `tb_market_items` (`item_key`) ON DELETE CASCADE;
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
